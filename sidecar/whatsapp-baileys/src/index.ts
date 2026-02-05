/**
 * Moltis WhatsApp Baileys Sidecar
 *
 * A WebSocket server that bridges the Moltis Rust gateway with WhatsApp Web via Baileys.
 * Supports multiple accounts, QR code login, inbound/outbound messages, reactions, and typing indicators.
 */

import { WebSocketServer, WebSocket } from "ws";
import makeWASocket, {
  DisconnectReason,
  useMultiFileAuthState,
  fetchLatestBaileysVersion,
  makeCacheableSignalKeyStore,
  type WASocket,
  type BaileysEventMap,
  type proto,
} from "@whiskeysockets/baileys";
import { Boom } from "@hapi/boom";
import pino from "pino";
import * as qrcode from "qrcode-terminal";
import * as fs from "fs";
import * as path from "path";
import type { GatewayMessage, SidecarMessage, AccountState } from "./types.js";

const PORT = parseInt(process.env.MOLTIS_WHATSAPP_PORT || "9876", 10);
const AUTH_BASE_DIR = process.env.MOLTIS_WHATSAPP_AUTH_DIR || path.join(process.env.HOME || ".", ".moltis", "whatsapp-auth");

const logger = pino({ level: process.env.LOG_LEVEL || "info" });

// Account state map
const accounts = new Map<string, AccountState>();

// Connected gateway clients
const clients = new Set<WebSocket>();

/**
 * Broadcast a message to all connected gateway clients.
 */
function broadcast(msg: SidecarMessage): void {
  const data = JSON.stringify(msg);
  for (const client of clients) {
    if (client.readyState === WebSocket.OPEN) {
      client.send(data);
    }
  }
}

/**
 * Extract text body from a WhatsApp message.
 */
function extractMessageBody(msg: proto.IWebMessageInfo): { body: string; mediaType?: string } {
  const m = msg.message;
  if (!m) return { body: "" };

  if (m.conversation) return { body: m.conversation };
  if (m.extendedTextMessage?.text) return { body: m.extendedTextMessage.text };
  if (m.imageMessage) return { body: m.imageMessage.caption || "", mediaType: "image" };
  if (m.videoMessage) return { body: m.videoMessage.caption || "", mediaType: "video" };
  if (m.audioMessage) return { body: "", mediaType: "audio" };
  if (m.documentMessage) return { body: m.documentMessage.caption || "", mediaType: "document" };
  if (m.stickerMessage) return { body: "", mediaType: "sticker" };
  if (m.locationMessage) {
    const loc = m.locationMessage;
    return { body: `Location: ${loc.degreesLatitude}, ${loc.degreesLongitude}`, mediaType: "location" };
  }
  if (m.contactMessage) return { body: m.contactMessage.displayName || "", mediaType: "contact" };
  if (m.buttonsResponseMessage) return { body: m.buttonsResponseMessage.selectedButtonId || "" };
  if (m.listResponseMessage) return { body: m.listResponseMessage.singleSelectReply?.selectedRowId || "" };
  if (m.templateButtonReplyMessage) return { body: m.templateButtonReplyMessage.selectedId || "" };
  if (m.reactionMessage) return { body: m.reactionMessage.text || "" };

  return { body: "" };
}

/**
 * Extract quoted message info if present.
 */
function extractQuotedInfo(msg: proto.IWebMessageInfo): { quotedMessageId?: string; quotedBody?: string } {
  const ctx = msg.message?.extendedTextMessage?.contextInfo;
  if (!ctx?.quotedMessage) return {};

  return {
    quotedMessageId: ctx.stanzaId || undefined,
    quotedBody: ctx.quotedMessage.conversation || ctx.quotedMessage.extendedTextMessage?.text || undefined,
  };
}

/**
 * Connect to WhatsApp for a specific account.
 */
async function connectAccount(accountId: string, authDir?: string): Promise<void> {
  const resolvedAuthDir = authDir || path.join(AUTH_BASE_DIR, accountId);

  // Ensure auth directory exists
  fs.mkdirSync(resolvedAuthDir, { recursive: true });

  const { state, saveCreds } = await useMultiFileAuthState(resolvedAuthDir);
  const { version } = await fetchLatestBaileysVersion();

  const sock = makeWASocket({
    version,
    logger: logger.child({ accountId }) as any,
    printQRInTerminal: false, // We handle QR ourselves
    auth: {
      creds: state.creds,
      keys: makeCacheableSignalKeyStore(state.keys, logger as any),
    },
    generateHighQualityLinkPreview: false,
    syncFullHistory: false,
    markOnlineOnConnect: true,
  });

  const accountState: AccountState = {
    accountId,
    authDir: resolvedAuthDir,
    connected: false,
    socket: sock,
  };
  accounts.set(accountId, accountState);

  // Handle connection updates
  sock.ev.on("connection.update", (update) => {
    const { connection, lastDisconnect, qr } = update;

    if (qr) {
      // Display QR in terminal for debugging
      qrcode.generate(qr, { small: true });
      // Send QR to gateway
      broadcast({ type: "qr", accountId, qr });
    }

    if (connection === "close") {
      const reason = (lastDisconnect?.error as Boom)?.output?.statusCode;
      const shouldReconnect = reason !== DisconnectReason.loggedOut;

      logger.info({ accountId, reason }, "Connection closed");

      if (reason === DisconnectReason.loggedOut) {
        // User logged out, clear credentials
        broadcast({ type: "logged_out", accountId });
        accounts.delete(accountId);
        // Clear auth files
        try {
          fs.rmSync(resolvedAuthDir, { recursive: true, force: true });
        } catch {
          // Ignore
        }
      } else {
        broadcast({
          type: "disconnected",
          accountId,
          reason: `Connection closed: ${reason || "unknown"}`,
        });

        if (shouldReconnect) {
          // Reconnect after a delay
          setTimeout(() => {
            if (accounts.has(accountId)) {
              connectAccount(accountId, authDir).catch((err) => {
                logger.error({ accountId, err }, "Failed to reconnect");
              });
            }
          }, 3000);
        }
      }
    }

    if (connection === "open") {
      accountState.connected = true;
      accountState.phoneNumber = sock.user?.id?.split(":")[0];
      broadcast({
        type: "connected",
        accountId,
        phoneNumber: accountState.phoneNumber,
      });
      logger.info({ accountId, phoneNumber: accountState.phoneNumber }, "Connected to WhatsApp");
    }
  });

  // Save credentials on update
  sock.ev.on("creds.update", saveCreds);

  // Handle inbound messages
  sock.ev.on("messages.upsert", async ({ messages, type }) => {
    // Only process new messages (not history sync)
    if (type !== "notify") return;

    for (const msg of messages) {
      // Skip status broadcasts
      if (msg.key.remoteJid === "status@broadcast") continue;
      // Skip our own messages
      if (msg.key.fromMe) continue;

      const chatJid = msg.key.remoteJid || "";
      const isGroup = chatJid.endsWith("@g.us");
      const senderJid = isGroup ? msg.key.participant || "" : chatJid;

      const { body, mediaType } = extractMessageBody(msg);
      const { quotedMessageId, quotedBody } = extractQuotedInfo(msg);

      // Get sender name from push name or contact
      const senderName = msg.pushName || undefined;

      broadcast({
        type: "inbound_message",
        accountId,
        messageId: msg.key.id || "",
        chatJid,
        senderJid,
        senderName,
        isGroup,
        body,
        mediaType,
        quotedMessageId,
        quotedBody,
        timestamp: msg.messageTimestamp
          ? typeof msg.messageTimestamp === "number"
            ? msg.messageTimestamp
            : Number(msg.messageTimestamp)
          : Date.now() / 1000,
      });
    }
  });
}

/**
 * Disconnect and remove an account.
 */
async function disconnectAccount(accountId: string): Promise<void> {
  const state = accounts.get(accountId);
  if (!state) return;

  if (state.socket) {
    state.socket.ev.removeAllListeners("connection.update");
    state.socket.ev.removeAllListeners("creds.update");
    state.socket.ev.removeAllListeners("messages.upsert");
    state.socket.end(undefined);
  }

  accounts.delete(accountId);
  broadcast({ type: "disconnected", accountId, reason: "manual disconnect" });
}

/**
 * Send a text message.
 */
async function sendText(
  accountId: string,
  to: string,
  text: string,
  requestId: string
): Promise<void> {
  const state = accounts.get(accountId);
  if (!state?.socket || !state.connected) {
    broadcast({ type: "send_result", requestId, success: false, error: "Not connected" });
    return;
  }

  try {
    // Chunk long messages (WhatsApp has ~65k limit but 4k is more readable)
    const chunks = chunkText(text, 4000);
    let lastMessageId: string | undefined;

    for (const chunk of chunks) {
      const result = await state.socket.sendMessage(to, { text: chunk });
      lastMessageId = result?.key?.id;
    }

    broadcast({ type: "send_result", requestId, success: true, messageId: lastMessageId });
  } catch (err) {
    logger.error({ accountId, to, err }, "Failed to send text");
    broadcast({
      type: "send_result",
      requestId,
      success: false,
      error: err instanceof Error ? err.message : String(err),
    });
  }
}

/**
 * Send a media message.
 */
async function sendMedia(
  accountId: string,
  to: string,
  mediaUrl: string,
  mediaType: string,
  caption: string | undefined,
  requestId: string
): Promise<void> {
  const state = accounts.get(accountId);
  if (!state?.socket || !state.connected) {
    broadcast({ type: "send_result", requestId, success: false, error: "Not connected" });
    return;
  }

  try {
    let content: Parameters<WASocket["sendMessage"]>[1];

    switch (mediaType) {
      case "image":
        content = { image: { url: mediaUrl }, caption };
        break;
      case "video":
        content = { video: { url: mediaUrl }, caption };
        break;
      case "audio":
        content = { audio: { url: mediaUrl }, ptt: true };
        break;
      case "document":
        content = { document: { url: mediaUrl }, caption };
        break;
      default:
        content = { document: { url: mediaUrl }, caption };
    }

    const result = await state.socket.sendMessage(to, content);
    broadcast({ type: "send_result", requestId, success: true, messageId: result?.key?.id });
  } catch (err) {
    logger.error({ accountId, to, mediaType, err }, "Failed to send media");
    broadcast({
      type: "send_result",
      requestId,
      success: false,
      error: err instanceof Error ? err.message : String(err),
    });
  }
}

/**
 * Send a reaction.
 */
async function sendReaction(
  accountId: string,
  chatJid: string,
  messageId: string,
  emoji: string,
  requestId: string
): Promise<void> {
  const state = accounts.get(accountId);
  if (!state?.socket || !state.connected) {
    broadcast({ type: "send_result", requestId, success: false, error: "Not connected" });
    return;
  }

  try {
    await state.socket.sendMessage(chatJid, {
      react: {
        text: emoji,
        key: { remoteJid: chatJid, id: messageId },
      },
    });
    broadcast({ type: "send_result", requestId, success: true });
  } catch (err) {
    logger.error({ accountId, chatJid, messageId, err }, "Failed to send reaction");
    broadcast({
      type: "send_result",
      requestId,
      success: false,
      error: err instanceof Error ? err.message : String(err),
    });
  }
}

/**
 * Send typing indicator.
 */
async function sendTyping(accountId: string, to: string): Promise<void> {
  const state = accounts.get(accountId);
  if (!state?.socket || !state.connected) return;

  try {
    await state.socket.sendPresenceUpdate("composing", to);
  } catch (err) {
    logger.warn({ accountId, to, err }, "Failed to send typing indicator");
  }
}

/**
 * Mark messages as read.
 */
async function markRead(accountId: string, chatJid: string, messageIds: string[]): Promise<void> {
  const state = accounts.get(accountId);
  if (!state?.socket || !state.connected) return;

  try {
    await state.socket.readMessages(
      messageIds.map((id) => ({ remoteJid: chatJid, id }))
    );
  } catch (err) {
    logger.warn({ accountId, chatJid, err }, "Failed to mark messages as read");
  }
}

/**
 * Get status of all accounts.
 */
function getStatus(): SidecarMessage {
  const accountList = Array.from(accounts.values()).map((state) => ({
    accountId: state.accountId,
    connected: state.connected,
    phoneNumber: state.phoneNumber,
    details: state.connected ? "Connected" : "Disconnected",
  }));

  return { type: "status_response", accounts: accountList };
}

/**
 * Chunk text for WhatsApp message limits.
 */
function chunkText(text: string, maxLen: number): string[] {
  if (text.length <= maxLen) return [text];

  const chunks: string[] = [];
  let remaining = text;

  while (remaining.length > 0) {
    if (remaining.length <= maxLen) {
      chunks.push(remaining);
      break;
    }

    // Try to break at newline
    let breakPoint = remaining.lastIndexOf("\n", maxLen);
    if (breakPoint === -1 || breakPoint < maxLen / 2) {
      // Try to break at space
      breakPoint = remaining.lastIndexOf(" ", maxLen);
    }
    if (breakPoint === -1 || breakPoint < maxLen / 2) {
      // Hard break
      breakPoint = maxLen;
    }

    chunks.push(remaining.slice(0, breakPoint));
    remaining = remaining.slice(breakPoint).trimStart();
  }

  return chunks;
}

/**
 * Handle incoming messages from the gateway.
 */
function handleGatewayMessage(ws: WebSocket, data: string): void {
  let msg: GatewayMessage;
  try {
    msg = JSON.parse(data);
  } catch (err) {
    logger.warn({ data }, "Invalid JSON from gateway");
    return;
  }

  switch (msg.type) {
    case "login":
      connectAccount(msg.accountId, msg.authDir).catch((err) => {
        logger.error({ accountId: msg.accountId, err }, "Failed to connect account");
        broadcast({ type: "error", accountId: msg.accountId, error: String(err) });
      });
      break;

    case "logout":
      disconnectAccount(msg.accountId).catch((err) => {
        logger.error({ accountId: msg.accountId, err }, "Failed to disconnect account");
      });
      break;

    case "status":
      ws.send(JSON.stringify(getStatus()));
      break;

    case "send_text":
      sendText(msg.accountId, msg.to, msg.text, msg.requestId);
      break;

    case "send_media":
      sendMedia(msg.accountId, msg.to, msg.mediaUrl, msg.mediaType, msg.caption, msg.requestId);
      break;

    case "send_reaction":
      sendReaction(msg.accountId, msg.chatJid, msg.messageId, msg.emoji, msg.requestId);
      break;

    case "send_typing":
      sendTyping(msg.accountId, msg.to);
      break;

    case "mark_read":
      markRead(msg.accountId, msg.chatJid, msg.messageIds);
      break;

    default:
      logger.warn({ msg }, "Unknown message type from gateway");
  }
}

// Start WebSocket server
const wss = new WebSocketServer({ port: PORT });

wss.on("connection", (ws) => {
  logger.info("Gateway client connected");
  clients.add(ws);

  ws.on("message", (data) => {
    handleGatewayMessage(ws, data.toString());
  });

  ws.on("close", () => {
    logger.info("Gateway client disconnected");
    clients.delete(ws);
  });

  ws.on("error", (err) => {
    logger.error({ err }, "WebSocket error");
    clients.delete(ws);
  });

  // Send current status to new client
  ws.send(JSON.stringify(getStatus()));
});

logger.info({ port: PORT, authDir: AUTH_BASE_DIR }, "WhatsApp Baileys sidecar started");

// Graceful shutdown
process.on("SIGINT", async () => {
  logger.info("Shutting down...");
  for (const [accountId] of accounts) {
    await disconnectAccount(accountId);
  }
  wss.close();
  process.exit(0);
});
