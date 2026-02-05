//! WhatsApp Cloud API types for webhook payloads.
//!
//! Reference: https://developers.facebook.com/docs/whatsapp/cloud-api/webhooks/payload-examples

use serde::{Deserialize, Serialize};

/// Root webhook payload from WhatsApp Cloud API.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookPayload {
    /// Should always be "whatsapp_business_account".
    pub object: String,
    /// List of changes/events.
    pub entry: Vec<WebhookEntry>,
}

/// A single entry in the webhook payload.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookEntry {
    /// WhatsApp Business Account ID.
    pub id: String,
    /// List of changes within this entry.
    pub changes: Vec<WebhookChange>,
}

/// A change within a webhook entry.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookChange {
    /// Field that changed (e.g., "messages").
    pub field: String,
    /// The actual change data.
    pub value: WebhookValue,
}

/// The value containing messages, statuses, and metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookValue {
    /// Messaging product (should be "whatsapp").
    pub messaging_product: Option<String>,
    /// Metadata about the WhatsApp Business phone number.
    pub metadata: Option<WebhookMetadata>,
    /// Contact information for the sender.
    #[serde(default)]
    pub contacts: Vec<WebhookContact>,
    /// Inbound messages.
    #[serde(default)]
    pub messages: Vec<InboundMessage>,
    /// Message delivery/read statuses.
    #[serde(default)]
    pub statuses: Vec<MessageStatus>,
    /// Errors, if any.
    #[serde(default)]
    pub errors: Vec<WebhookError>,
}

/// Metadata about the receiving WhatsApp Business phone number.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookMetadata {
    /// Display phone number (e.g., "+1 555 123 4567").
    pub display_phone_number: String,
    /// Phone number ID used for API calls.
    pub phone_number_id: String,
}

/// Contact information for the message sender.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookContact {
    /// WhatsApp ID (phone number without + prefix).
    pub wa_id: String,
    /// Profile information.
    pub profile: Option<ContactProfile>,
}

/// Profile information for a contact.
#[derive(Debug, Clone, Deserialize)]
pub struct ContactProfile {
    /// Display name set by the user.
    pub name: String,
}

/// An inbound message from WhatsApp.
#[derive(Debug, Clone, Deserialize)]
pub struct InboundMessage {
    /// Sender's WhatsApp ID (phone number).
    pub from: String,
    /// Unique message ID.
    pub id: String,
    /// Unix timestamp of the message.
    pub timestamp: String,
    /// Message type: text, image, document, audio, video, sticker, location, contacts, interactive, button, reaction.
    #[serde(rename = "type")]
    pub message_type: String,
    /// Text message content.
    pub text: Option<TextContent>,
    /// Image content.
    pub image: Option<MediaContent>,
    /// Document content.
    pub document: Option<DocumentContent>,
    /// Audio content.
    pub audio: Option<MediaContent>,
    /// Video content.
    pub video: Option<MediaContent>,
    /// Sticker content.
    pub sticker: Option<MediaContent>,
    /// Location content.
    pub location: Option<LocationContent>,
    /// Interactive message response (list/button replies).
    pub interactive: Option<InteractiveContent>,
    /// Button reply content.
    pub button: Option<ButtonContent>,
    /// Reaction content.
    pub reaction: Option<ReactionContent>,
    /// Context (reply-to information).
    pub context: Option<MessageContext>,
}

impl InboundMessage {
    /// Extract the text body from this message.
    pub fn text_body(&self) -> Option<String> {
        match self.message_type.as_str() {
            "text" => self.text.as_ref().map(|t| t.body.clone()),
            "image" => self.image.as_ref().and_then(|i| i.caption.clone()),
            "document" => self.document.as_ref().and_then(|d| d.caption.clone()),
            "audio" => self.audio.as_ref().and_then(|a| a.caption.clone()),
            "video" => self.video.as_ref().and_then(|v| v.caption.clone()),
            "interactive" => self.interactive.as_ref().map(|i| i.extract_text()),
            "button" => self.button.as_ref().map(|b| b.text.clone()),
            _ => None,
        }
    }

    /// Check if this message has media content.
    pub fn has_media(&self) -> bool {
        matches!(
            self.message_type.as_str(),
            "image" | "document" | "audio" | "video" | "sticker"
        )
    }

    /// Get the media ID if this message contains media.
    pub fn media_id(&self) -> Option<&str> {
        match self.message_type.as_str() {
            "image" => self.image.as_ref().map(|i| i.id.as_str()),
            "document" => self.document.as_ref().map(|d| d.id.as_str()),
            "audio" => self.audio.as_ref().map(|a| a.id.as_str()),
            "video" => self.video.as_ref().map(|v| v.id.as_str()),
            "sticker" => self.sticker.as_ref().map(|s| s.id.as_str()),
            _ => None,
        }
    }
}

/// Text message content.
#[derive(Debug, Clone, Deserialize)]
pub struct TextContent {
    /// The message text.
    pub body: String,
    /// Whether this is a preview URL message.
    #[serde(default)]
    pub preview_url: bool,
}

/// Media content (image, audio, video, sticker).
#[derive(Debug, Clone, Deserialize)]
pub struct MediaContent {
    /// Media ID for downloading via the API.
    pub id: String,
    /// MIME type of the media.
    pub mime_type: Option<String>,
    /// SHA256 hash of the media.
    pub sha256: Option<String>,
    /// Caption (for images/videos).
    pub caption: Option<String>,
}

/// Document content.
#[derive(Debug, Clone, Deserialize)]
pub struct DocumentContent {
    /// Media ID for downloading via the API.
    pub id: String,
    /// MIME type of the document.
    pub mime_type: Option<String>,
    /// SHA256 hash of the document.
    pub sha256: Option<String>,
    /// Filename of the document.
    pub filename: Option<String>,
    /// Caption.
    pub caption: Option<String>,
}

/// Location content.
#[derive(Debug, Clone, Deserialize)]
pub struct LocationContent {
    /// Latitude.
    pub latitude: f64,
    /// Longitude.
    pub longitude: f64,
    /// Location name.
    pub name: Option<String>,
    /// Address.
    pub address: Option<String>,
}

/// Interactive message response (button/list replies).
#[derive(Debug, Clone, Deserialize)]
pub struct InteractiveContent {
    /// Type of interactive response: "button_reply" or "list_reply".
    #[serde(rename = "type")]
    pub interactive_type: String,
    /// Button reply data.
    pub button_reply: Option<ButtonReply>,
    /// List reply data.
    pub list_reply: Option<ListReply>,
}

impl InteractiveContent {
    /// Extract the text from this interactive response.
    pub fn extract_text(&self) -> String {
        if let Some(ref br) = self.button_reply {
            return br.title.clone();
        }
        if let Some(ref lr) = self.list_reply {
            return lr.title.clone();
        }
        String::new()
    }
}

/// Button reply data.
#[derive(Debug, Clone, Deserialize)]
pub struct ButtonReply {
    /// Button ID.
    pub id: String,
    /// Button title/text.
    pub title: String,
}

/// List reply data.
#[derive(Debug, Clone, Deserialize)]
pub struct ListReply {
    /// Selected item ID.
    pub id: String,
    /// Selected item title.
    pub title: String,
    /// Selected item description.
    pub description: Option<String>,
}

/// Button content (quick reply buttons).
#[derive(Debug, Clone, Deserialize)]
pub struct ButtonContent {
    /// Button payload.
    pub payload: String,
    /// Button text.
    pub text: String,
}

/// Reaction content.
#[derive(Debug, Clone, Deserialize)]
pub struct ReactionContent {
    /// Message ID being reacted to.
    pub message_id: String,
    /// Emoji reaction (empty string means reaction removed).
    pub emoji: String,
}

/// Reply-to context.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageContext {
    /// ID of the message being replied to.
    pub id: Option<String>,
    /// Sender of the message being replied to.
    pub from: Option<String>,
}

/// Message delivery/read status.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageStatus {
    /// Message ID.
    pub id: String,
    /// Recipient's WhatsApp ID.
    pub recipient_id: String,
    /// Status: "sent", "delivered", "read", "failed".
    pub status: String,
    /// Unix timestamp.
    pub timestamp: String,
    /// Errors, if status is "failed".
    #[serde(default)]
    pub errors: Vec<WebhookError>,
}

/// Webhook error.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookError {
    /// Error code.
    pub code: i32,
    /// Error title.
    pub title: Option<String>,
    /// Error message.
    pub message: Option<String>,
    /// Error details.
    pub error_data: Option<serde_json::Value>,
}

// ── Outbound message types ────────────────────────────────────────────────────

/// Request body for sending a text message.
#[derive(Debug, Clone, Serialize)]
pub struct SendTextRequest {
    /// Must be "whatsapp".
    pub messaging_product: &'static str,
    /// Recipient type (usually "individual").
    pub recipient_type: &'static str,
    /// Recipient's WhatsApp ID (phone number).
    pub to: String,
    /// Message type.
    #[serde(rename = "type")]
    pub message_type: &'static str,
    /// Text content.
    pub text: OutboundTextContent,
}

impl SendTextRequest {
    pub fn new(to: String, body: String) -> Self {
        Self {
            messaging_product: "whatsapp",
            recipient_type: "individual",
            to,
            message_type: "text",
            text: OutboundTextContent {
                preview_url: false,
                body,
            },
        }
    }
}

/// Outbound text content.
#[derive(Debug, Clone, Serialize)]
pub struct OutboundTextContent {
    /// Whether to preview URLs.
    pub preview_url: bool,
    /// Message body.
    pub body: String,
}

/// Request body for sending a media message.
#[derive(Debug, Clone, Serialize)]
pub struct SendMediaRequest {
    /// Must be "whatsapp".
    pub messaging_product: &'static str,
    /// Recipient type.
    pub recipient_type: &'static str,
    /// Recipient's WhatsApp ID.
    pub to: String,
    /// Message type: "image", "document", "audio", "video".
    #[serde(rename = "type")]
    pub message_type: String,
    /// Media content (field name matches message_type).
    #[serde(flatten)]
    pub media: OutboundMediaContent,
}

/// Outbound media content.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum OutboundMediaContent {
    Image { image: MediaObject },
    Document { document: DocumentObject },
    Audio { audio: MediaObject },
    Video { video: MediaObject },
}

/// Media object for outbound messages.
#[derive(Debug, Clone, Serialize)]
pub struct MediaObject {
    /// Media URL (for URL-based sending).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// Media ID (for ID-based sending).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Caption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

/// Document object for outbound messages.
#[derive(Debug, Clone, Serialize)]
pub struct DocumentObject {
    /// Document URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    /// Document ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Caption.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    /// Filename.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// Response from the send message API.
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageResponse {
    /// Messaging product.
    pub messaging_product: String,
    /// Contacts (recipients).
    pub contacts: Vec<SendMessageContact>,
    /// Sent messages.
    pub messages: Vec<SentMessage>,
}

/// Contact in send message response.
#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageContact {
    /// Input phone number.
    pub input: String,
    /// WhatsApp ID.
    pub wa_id: String,
}

/// Sent message info.
#[derive(Debug, Clone, Deserialize)]
pub struct SentMessage {
    /// Message ID.
    pub id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_message_webhook() {
        let json = r#"{
            "object": "whatsapp_business_account",
            "entry": [{
                "id": "123456789",
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messaging_product": "whatsapp",
                        "metadata": {
                            "display_phone_number": "+1 555 123 4567",
                            "phone_number_id": "123456789"
                        },
                        "contacts": [{
                            "wa_id": "15551234567",
                            "profile": {"name": "John Doe"}
                        }],
                        "messages": [{
                            "from": "15551234567",
                            "id": "wamid.xxx",
                            "timestamp": "1234567890",
                            "type": "text",
                            "text": {"body": "Hello, world!"}
                        }]
                    }
                }]
            }]
        }"#;

        let payload: WebhookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.object, "whatsapp_business_account");
        assert_eq!(payload.entry.len(), 1);

        let change = &payload.entry[0].changes[0];
        assert_eq!(change.field, "messages");

        let messages = &change.value.messages;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].from, "15551234567");
        assert_eq!(messages[0].text_body(), Some("Hello, world!".to_string()));
    }

    #[test]
    fn parse_image_message_webhook() {
        let json = r#"{
            "object": "whatsapp_business_account",
            "entry": [{
                "id": "123",
                "changes": [{
                    "field": "messages",
                    "value": {
                        "messages": [{
                            "from": "15551234567",
                            "id": "wamid.xxx",
                            "timestamp": "1234567890",
                            "type": "image",
                            "image": {
                                "id": "media123",
                                "mime_type": "image/jpeg",
                                "sha256": "abc123",
                                "caption": "Check this out!"
                            }
                        }]
                    }
                }]
            }]
        }"#;

        let payload: WebhookPayload = serde_json::from_str(json).unwrap();
        let msg = &payload.entry[0].changes[0].value.messages[0];
        assert!(msg.has_media());
        assert_eq!(msg.media_id(), Some("media123"));
        assert_eq!(msg.text_body(), Some("Check this out!".to_string()));
    }

    #[test]
    fn serialize_send_text_request() {
        let req = SendTextRequest::new("15551234567".into(), "Hello!".into());
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"messaging_product\":\"whatsapp\""));
        assert!(json.contains("\"to\":\"15551234567\""));
        assert!(json.contains("\"body\":\"Hello!\""));
    }
}
