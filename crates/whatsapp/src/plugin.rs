//! WhatsApp channel plugin implementation.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use {
    anyhow::Result,
    async_trait::async_trait,
    secrecy::ExposeSecret,
    tracing::{info, warn},
};

use moltis_channels::{
    ChannelEventSink,
    message_log::MessageLog,
    plugin::{ChannelHealthSnapshot, ChannelOutbound, ChannelPlugin, ChannelStatus},
};

use crate::{
    config::WhatsAppAccountConfig,
    outbound::WhatsAppOutbound,
    state::{AccountState, AccountStateMap},
};

/// WhatsApp channel plugin.
pub struct WhatsAppPlugin {
    accounts: AccountStateMap,
    outbound: WhatsAppOutbound,
    message_log: Option<Arc<dyn MessageLog>>,
    event_sink: Option<Arc<dyn ChannelEventSink>>,
}

impl WhatsAppPlugin {
    pub fn new() -> Self {
        let accounts: AccountStateMap = Arc::new(RwLock::new(HashMap::new()));
        let outbound = WhatsAppOutbound::new(Arc::clone(&accounts));
        Self {
            accounts,
            outbound,
            message_log: None,
            event_sink: None,
        }
    }

    pub fn with_message_log(mut self, log: Arc<dyn MessageLog>) -> Self {
        self.message_log = Some(log);
        self
    }

    pub fn with_event_sink(mut self, sink: Arc<dyn ChannelEventSink>) -> Self {
        self.event_sink = Some(sink);
        self
    }

    /// Get a shared reference to the outbound sender (for use outside the plugin).
    pub fn shared_outbound(&self) -> Arc<dyn ChannelOutbound> {
        Arc::new(WhatsAppOutbound::new(Arc::clone(&self.accounts)))
    }

    /// Get the shared account state map (for webhook handlers).
    pub fn accounts(&self) -> AccountStateMap {
        Arc::clone(&self.accounts)
    }

    /// List all active account IDs.
    pub fn account_ids(&self) -> Vec<String> {
        let accounts = self.accounts.read().unwrap();
        accounts.keys().cloned().collect()
    }

    /// Get the config for a specific account (serialized to JSON).
    pub fn account_config(&self, account_id: &str) -> Option<serde_json::Value> {
        let accounts = self.accounts.read().unwrap();
        accounts
            .get(account_id)
            .and_then(|s| serde_json::to_value(&s.config).ok())
    }

    /// Get the config for a specific account.
    pub fn get_account_config(&self, account_id: &str) -> Option<WhatsAppAccountConfig> {
        let accounts = self.accounts.read().unwrap();
        accounts.get(account_id).map(|s| s.config.clone())
    }
}

impl Default for WhatsAppPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChannelPlugin for WhatsAppPlugin {
    fn id(&self) -> &str {
        "whatsapp"
    }

    fn name(&self) -> &str {
        "WhatsApp"
    }

    async fn start_account(&mut self, account_id: &str, config: serde_json::Value) -> Result<()> {
        let wa_config: WhatsAppAccountConfig = serde_json::from_value(config)?;

        if wa_config.phone_number_id.is_empty() {
            return Err(anyhow::anyhow!("whatsapp phone_number_id is required"));
        }

        if wa_config.access_token.expose_secret().is_empty() {
            return Err(anyhow::anyhow!("whatsapp access_token is required"));
        }

        if wa_config.app_secret.expose_secret().is_empty() {
            return Err(anyhow::anyhow!("whatsapp app_secret is required"));
        }

        info!(account_id, phone_number_id = %wa_config.phone_number_id, "starting whatsapp account");

        // Create HTTP client for API calls.
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let outbound = Arc::new(WhatsAppOutbound::new(Arc::clone(&self.accounts)));

        let state = AccountState {
            account_id: account_id.to_string(),
            config: wa_config,
            outbound,
            message_log: self.message_log.clone(),
            event_sink: self.event_sink.clone(),
            http_client,
        };

        {
            let mut accounts = self.accounts.write().unwrap();
            accounts.insert(account_id.to_string(), state);
        }

        Ok(())
    }

    async fn stop_account(&mut self, account_id: &str) -> Result<()> {
        let removed = {
            let mut accounts = self.accounts.write().unwrap();
            accounts.remove(account_id).is_some()
        };

        if removed {
            info!(account_id, "stopped whatsapp account");
        } else {
            warn!(account_id, "whatsapp account not found");
        }

        Ok(())
    }

    fn outbound(&self) -> Option<&dyn ChannelOutbound> {
        Some(&self.outbound)
    }

    fn status(&self) -> Option<&dyn ChannelStatus> {
        Some(self)
    }
}

#[async_trait]
impl ChannelStatus for WhatsAppPlugin {
    async fn probe(&self, account_id: &str) -> Result<ChannelHealthSnapshot> {
        let config = {
            let accounts = self.accounts.read().unwrap();
            accounts.get(account_id).map(|s| s.config.clone())
        };

        match config {
            Some(cfg) => {
                // For WhatsApp, we consider the account "connected" if it's configured.
                // A more sophisticated check could call the Graph API to verify the token.
                Ok(ChannelHealthSnapshot {
                    connected: true,
                    account_id: account_id.to_string(),
                    details: Some(format!("Phone: {}", cfg.phone_number_id)),
                })
            },
            None => Ok(ChannelHealthSnapshot {
                connected: false,
                account_id: account_id.to_string(),
                details: Some("account not started".into()),
            }),
        }
    }
}
