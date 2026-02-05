/**
 * Messages sent from the Rust gateway to the sidecar.
 */
export type GatewayMessage =
  | { type: "login"; accountId: string; authDir?: string }
  | { type: "logout"; accountId: string }
  | { type: "status"; accountId?: string }
  | { type: "send_text"; accountId: string; to: string; text: string; requestId: string }
  | {
      type: "send_media";
      accountId: string;
      to: string;
      mediaUrl: string;
      mediaType: string;
      caption?: string;
      requestId: string;
    }
  | { type: "send_reaction"; accountId: string; chatJid: string; messageId: string; emoji: string; requestId: string }
  | { type: "send_typing"; accountId: string; to: string }
  | { type: "mark_read"; accountId: string; chatJid: string; messageIds: string[] };

/**
 * Messages sent from the sidecar to the Rust gateway.
 */
export type SidecarMessage =
  | { type: "qr"; accountId: string; qr: string }
  | { type: "connected"; accountId: string; phoneNumber?: string }
  | { type: "disconnected"; accountId: string; reason: string }
  | { type: "logged_out"; accountId: string }
  | {
      type: "inbound_message";
      accountId: string;
      messageId: string;
      chatJid: string;
      senderJid: string;
      senderName?: string;
      isGroup: boolean;
      body: string;
      mediaType?: string;
      mediaUrl?: string;
      quotedMessageId?: string;
      quotedBody?: string;
      timestamp: number;
    }
  | { type: "send_result"; requestId: string; success: boolean; messageId?: string; error?: string }
  | {
      type: "status_response";
      accounts: Array<{
        accountId: string;
        connected: boolean;
        phoneNumber?: string;
        details?: string;
      }>;
    }
  | { type: "error"; accountId?: string; error: string };

/**
 * Per-account state.
 */
export type AccountState = {
  accountId: string;
  authDir: string;
  connected: boolean;
  phoneNumber?: string;
  socket?: import("@whiskeysockets/baileys").WASocket;
};
