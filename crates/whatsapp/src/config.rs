//! WhatsApp Web account configuration.

use serde::{Deserialize, Serialize};

/// Configuration for a WhatsApp Web account (via Baileys sidecar).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConfig {
    /// Directory to store authentication state (optional, defaults to ~/.moltis/whatsapp-auth/<account_id>).
    #[serde(default)]
    pub auth_dir: Option<String>,

    /// Allowlist of phone numbers that can interact with the bot (empty = allow all).
    #[serde(default)]
    pub allowlist: Vec<String>,

    /// Default model to use for this channel account.
    #[serde(default)]
    pub model: Option<String>,

    /// Whether to auto-mark messages as read.
    #[serde(default = "default_auto_read")]
    pub auto_read: bool,
}

fn default_auto_read() -> bool {
    true
}

impl Default for WhatsAppConfig {
    fn default() -> Self {
        Self {
            auth_dir: None,
            allowlist: Vec::new(),
            model: None,
            auto_read: true,
        }
    }
}
