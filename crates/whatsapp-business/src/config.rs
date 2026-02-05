//! WhatsApp channel configuration.

use {
    moltis_channels::gating::{DmPolicy, GroupPolicy},
    secrecy::{ExposeSecret, Secret},
    serde::{Deserialize, Serialize},
};

/// Configuration for a single WhatsApp Business account.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WhatsAppAccountConfig {
    /// WhatsApp Business Phone Number ID (from Meta Business Suite).
    pub phone_number_id: String,

    /// WhatsApp Business Account access token.
    #[serde(serialize_with = "serialize_secret")]
    pub access_token: Secret<String>,

    /// App secret for webhook signature verification.
    #[serde(serialize_with = "serialize_secret")]
    pub app_secret: Secret<String>,

    /// Webhook verification token (used during webhook registration).
    pub verify_token: String,

    /// WhatsApp Business Account ID (optional, for display purposes).
    pub business_account_id: Option<String>,

    /// DM access policy.
    pub dm_policy: DmPolicy,

    /// Group access policy (WhatsApp groups).
    pub group_policy: GroupPolicy,

    /// User/phone number allowlist for DMs.
    pub allowlist: Vec<String>,

    /// Group ID allowlist.
    pub group_allowlist: Vec<String>,

    /// Default model ID for this account's sessions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// Provider name associated with `model`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_provider: Option<String>,

    /// Base URL for the WhatsApp Cloud API. Defaults to the official Meta Graph API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,
}

impl WhatsAppAccountConfig {
    /// Get the API base URL, defaulting to the official Meta Graph API.
    pub fn api_base_url(&self) -> &str {
        self.api_base_url
            .as_deref()
            .unwrap_or("https://graph.facebook.com/v21.0")
    }

    /// Build the messages API endpoint URL.
    pub fn messages_url(&self) -> String {
        format!("{}/{}/messages", self.api_base_url(), self.phone_number_id)
    }

    /// Build the media API endpoint URL for downloading media.
    pub fn media_url(&self, media_id: &str) -> String {
        format!("{}/{}", self.api_base_url(), media_id)
    }
}

impl std::fmt::Debug for WhatsAppAccountConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WhatsAppAccountConfig")
            .field("phone_number_id", &self.phone_number_id)
            .field("access_token", &"[REDACTED]")
            .field("app_secret", &"[REDACTED]")
            .field("verify_token", &self.verify_token)
            .field("dm_policy", &self.dm_policy)
            .field("group_policy", &self.group_policy)
            .finish_non_exhaustive()
    }
}

fn serialize_secret<S: serde::Serializer>(
    secret: &Secret<String>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(secret.expose_secret())
}

impl Default for WhatsAppAccountConfig {
    fn default() -> Self {
        Self {
            phone_number_id: String::new(),
            access_token: Secret::new(String::new()),
            app_secret: Secret::new(String::new()),
            verify_token: String::new(),
            business_account_id: None,
            dm_policy: DmPolicy::default(),
            group_policy: GroupPolicy::default(),
            allowlist: Vec::new(),
            group_allowlist: Vec::new(),
            model: None,
            model_provider: None,
            api_base_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = WhatsAppAccountConfig::default();
        assert_eq!(cfg.dm_policy, DmPolicy::Open);
        assert_eq!(cfg.group_policy, GroupPolicy::Open);
        assert!(cfg.phone_number_id.is_empty());
    }

    #[test]
    fn deserialize_from_json() {
        let json = r#"{
            "phone_number_id": "123456789",
            "access_token": "EAAxxxx",
            "app_secret": "abc123",
            "verify_token": "mytoken",
            "dm_policy": "allowlist",
            "allowlist": ["15551234567"]
        }"#;
        let cfg: WhatsAppAccountConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.phone_number_id, "123456789");
        assert_eq!(cfg.access_token.expose_secret(), "EAAxxxx");
        assert_eq!(cfg.app_secret.expose_secret(), "abc123");
        assert_eq!(cfg.verify_token, "mytoken");
        assert_eq!(cfg.dm_policy, DmPolicy::Allowlist);
        assert_eq!(cfg.allowlist, vec!["15551234567"]);
        // defaults for unspecified fields
        assert_eq!(cfg.group_policy, GroupPolicy::Open);
    }

    #[test]
    fn serialize_roundtrip() {
        let cfg = WhatsAppAccountConfig {
            phone_number_id: "123".into(),
            access_token: Secret::new("tok".into()),
            app_secret: Secret::new("sec".into()),
            verify_token: "ver".into(),
            dm_policy: DmPolicy::Disabled,
            ..Default::default()
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let cfg2: WhatsAppAccountConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg2.dm_policy, DmPolicy::Disabled);
        assert_eq!(cfg2.access_token.expose_secret(), "tok");
    }

    #[test]
    fn api_urls() {
        let cfg = WhatsAppAccountConfig {
            phone_number_id: "123456789".into(),
            ..Default::default()
        };
        assert_eq!(
            cfg.messages_url(),
            "https://graph.facebook.com/v21.0/123456789/messages"
        );
        assert_eq!(
            cfg.media_url("media_abc"),
            "https://graph.facebook.com/v21.0/media_abc"
        );
    }
}
