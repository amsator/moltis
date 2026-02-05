//! Types for communication with the WhatsApp Baileys sidecar.

use serde::{Deserialize, Serialize};

/// Messages sent from Rust to the sidecar.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayMessage {
    Login {
        #[serde(rename = "accountId")]
        account_id: String,
        #[serde(rename = "authDir", skip_serializing_if = "Option::is_none")]
        auth_dir: Option<String>,
    },
    Logout {
        #[serde(rename = "accountId")]
        account_id: String,
    },
    Status {
        #[serde(rename = "accountId", skip_serializing_if = "Option::is_none")]
        account_id: Option<String>,
    },
    SendText {
        #[serde(rename = "accountId")]
        account_id: String,
        to: String,
        text: String,
        #[serde(rename = "requestId")]
        request_id: String,
    },
    SendMedia {
        #[serde(rename = "accountId")]
        account_id: String,
        to: String,
        #[serde(rename = "mediaUrl")]
        media_url: String,
        #[serde(rename = "mediaType")]
        media_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        caption: Option<String>,
        #[serde(rename = "requestId")]
        request_id: String,
    },
    SendReaction {
        #[serde(rename = "accountId")]
        account_id: String,
        #[serde(rename = "chatJid")]
        chat_jid: String,
        #[serde(rename = "messageId")]
        message_id: String,
        emoji: String,
        #[serde(rename = "requestId")]
        request_id: String,
    },
    SendTyping {
        #[serde(rename = "accountId")]
        account_id: String,
        to: String,
    },
    MarkRead {
        #[serde(rename = "accountId")]
        account_id: String,
        #[serde(rename = "chatJid")]
        chat_jid: String,
        #[serde(rename = "messageIds")]
        message_ids: Vec<String>,
    },
}

/// Messages received from the sidecar.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SidecarMessage {
    Qr {
        #[serde(rename = "accountId")]
        account_id: String,
        qr: String,
    },
    Connected {
        #[serde(rename = "accountId")]
        account_id: String,
        #[serde(rename = "phoneNumber")]
        phone_number: Option<String>,
    },
    Disconnected {
        #[serde(rename = "accountId")]
        account_id: String,
        reason: String,
    },
    LoggedOut {
        #[serde(rename = "accountId")]
        account_id: String,
    },
    InboundMessage {
        #[serde(rename = "accountId")]
        account_id: String,
        #[serde(rename = "messageId")]
        message_id: String,
        #[serde(rename = "chatJid")]
        chat_jid: String,
        #[serde(rename = "senderJid")]
        sender_jid: String,
        #[serde(rename = "senderName")]
        sender_name: Option<String>,
        #[serde(rename = "isGroup")]
        is_group: bool,
        body: String,
        #[serde(rename = "mediaType")]
        media_type: Option<String>,
        #[serde(rename = "mediaUrl")]
        media_url: Option<String>,
        #[serde(rename = "quotedMessageId")]
        quoted_message_id: Option<String>,
        #[serde(rename = "quotedBody")]
        quoted_body: Option<String>,
        timestamp: f64,
    },
    SendResult {
        #[serde(rename = "requestId")]
        request_id: String,
        success: bool,
        #[serde(rename = "messageId")]
        message_id: Option<String>,
        error: Option<String>,
    },
    StatusResponse {
        accounts: Vec<AccountStatus>,
    },
    Error {
        #[serde(rename = "accountId")]
        account_id: Option<String>,
        error: String,
    },
}

/// Account status from the sidecar.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountStatus {
    #[serde(rename = "accountId")]
    pub account_id: String,
    pub connected: bool,
    #[serde(rename = "phoneNumber")]
    pub phone_number: Option<String>,
    pub details: Option<String>,
}

/// Connection state for a WhatsApp account.
#[derive(Debug, Clone, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    WaitingForQr,
    QrReceived(String),
    Connected {
        phone_number: Option<String>,
    },
}
