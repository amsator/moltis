//! WhatsApp Web channel plugin implementation.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock as StdRwLock},
};

use {
    anyhow::Result,
    async_trait::async_trait,
    tokio::sync::RwLock,
    tracing::{debug, info, warn},
};

use moltis_channels::{
    ChannelEventSink,
    message_log::MessageLog,
    plugin::{
        ChannelEvent, ChannelHealthSnapshot, ChannelMessageMeta, ChannelOutbound, ChannelPlugin,
        ChannelReplyTarget, ChannelStatus,
    },
};

use crate::{
    config::WhatsAppConfig,
    outbound::WhatsAppOutbound,
    sidecar::{DEFAULT_SIDECAR_PORT, MessageCallback, SidecarHandle, connect_to_sidecar},
    state::{AccountState, AccountStateMap},
    types::{ConnectionState, GatewayMessage, SidecarMessage},
};

/// WhatsApp Web channel plugin (via Baileys sidecar).
pub struct WhatsAppPlugin {
    accounts: AccountStateMap,
    outbound: WhatsAppOutbound,
    sidecar: Arc<RwLock<Option<SidecarHandle>>>,
    message_log: Option<Arc<dyn MessageLog>>,
    event_sink: Option<Arc<dyn ChannelEventSink>>,
    sidecar_port: u16,
}

impl WhatsAppPlugin {
    pub fn new() -> Self {
        let sidecar: Arc<RwLock<Option<SidecarHandle>>> = Arc::new(RwLock::new(None));
        let outbound = WhatsAppOutbound::new(Arc::clone(&sidecar));
        Self {
            accounts: Arc::new(StdRwLock::new(HashMap::new())),
            outbound,
            sidecar,
            message_log: None,
            event_sink: None,
            sidecar_port: DEFAULT_SIDECAR_PORT,
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

    pub fn with_sidecar_port(mut self, port: u16) -> Self {
        self.sidecar_port = port;
        self
    }

    /// Get a shared reference to the outbound sender.
    pub fn shared_outbound(&self) -> Arc<dyn ChannelOutbound> {
        Arc::new(WhatsAppOutbound::new(Arc::clone(&self.sidecar)))
    }

    /// Get the shared account state map.
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
    pub fn get_account_config(&self, account_id: &str) -> Option<WhatsAppConfig> {
        let accounts = self.accounts.read().unwrap();
        accounts.get(account_id).map(|s| s.config.clone())
    }

    /// Get the connection state for an account.
    pub fn connection_state(&self, account_id: &str) -> Option<ConnectionState> {
        let accounts = self.accounts.read().unwrap();
        accounts.get(account_id).map(|s| s.connection_state.clone())
    }

    /// Get the current QR code for an account (if waiting for scan).
    pub fn get_qr_code(&self, account_id: &str) -> Option<String> {
        let accounts = self.accounts.read().unwrap();
        accounts.get(account_id).and_then(|s| {
            if let ConnectionState::QrReceived(qr) = &s.connection_state {
                Some(qr.clone())
            } else {
                None
            }
        })
    }

    /// Connect to the sidecar process.
    async fn ensure_sidecar_connected(&self) -> Result<()> {
        let mut sidecar = self.sidecar.write().await;
        if sidecar.is_some() {
            return Ok(());
        }

        let accounts = Arc::clone(&self.accounts);
        let event_sink = self.event_sink.clone();
        let message_log = self.message_log.clone();

        let callback: MessageCallback = Arc::new(move |msg| {
            handle_sidecar_message(
                msg,
                Arc::clone(&accounts),
                event_sink.clone(),
                message_log.clone(),
            );
        });

        let (handle, _disconnect_rx) = connect_to_sidecar(self.sidecar_port, callback).await?;
        *sidecar = Some(handle);

        Ok(())
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
        "whatsapp-web"
    }

    fn name(&self) -> &str {
        "WhatsApp Web"
    }

    async fn start_account(&mut self, account_id: &str, config: serde_json::Value) -> Result<()> {
        let wa_config: WhatsAppConfig = serde_json::from_value(config)?;

        info!(account_id, "starting whatsapp web account");

        // Store account state.
        let state = AccountState {
            account_id: account_id.to_string(),
            config: wa_config.clone(),
            connection_state: ConnectionState::Disconnected,
            message_log: self.message_log.clone(),
            event_sink: self.event_sink.clone(),
        };

        {
            let mut accounts = self.accounts.write().unwrap();
            accounts.insert(account_id.to_string(), state);
        }

        // Connect to sidecar if not already connected.
        if let Err(e) = self.ensure_sidecar_connected().await {
            warn!(account_id, error = %e, "failed to connect to sidecar");
            // Update state to show error.
            let mut accounts = self.accounts.write().unwrap();
            if let Some(state) = accounts.get_mut(account_id) {
                state.connection_state = ConnectionState::Disconnected;
            }
            return Err(e);
        }

        // Tell sidecar to login.
        let sidecar = self.sidecar.read().await;
        if let Some(handle) = sidecar.as_ref() {
            handle
                .send(GatewayMessage::Login {
                    account_id: account_id.to_string(),
                    auth_dir: wa_config.auth_dir,
                })
                .await?;

            // Mark as waiting for QR.
            let mut accounts = self.accounts.write().unwrap();
            if let Some(state) = accounts.get_mut(account_id) {
                state.connection_state = ConnectionState::WaitingForQr;
            }
        }

        Ok(())
    }

    async fn stop_account(&mut self, account_id: &str) -> Result<()> {
        let removed = {
            let mut accounts = self.accounts.write().unwrap();
            accounts.remove(account_id).is_some()
        };

        if removed {
            // Tell sidecar to logout.
            let sidecar = self.sidecar.read().await;
            if let Some(handle) = sidecar.as_ref() {
                let _ = handle
                    .send(GatewayMessage::Logout {
                        account_id: account_id.to_string(),
                    })
                    .await;
            }
            info!(account_id, "stopped whatsapp web account");
        } else {
            warn!(account_id, "whatsapp web account not found");
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
        let state = {
            let accounts = self.accounts.read().unwrap();
            accounts.get(account_id).map(|s| s.connection_state.clone())
        };

        match state {
            Some(ConnectionState::Connected { phone_number }) => Ok(ChannelHealthSnapshot {
                connected: true,
                account_id: account_id.to_string(),
                details: phone_number.map(|p| format!("Phone: {p}")),
            }),
            Some(ConnectionState::QrReceived(_)) => Ok(ChannelHealthSnapshot {
                connected: false,
                account_id: account_id.to_string(),
                details: Some("waiting for QR code scan".into()),
            }),
            Some(ConnectionState::WaitingForQr) => Ok(ChannelHealthSnapshot {
                connected: false,
                account_id: account_id.to_string(),
                details: Some("generating QR code".into()),
            }),
            Some(ConnectionState::Disconnected) | None => Ok(ChannelHealthSnapshot {
                connected: false,
                account_id: account_id.to_string(),
                details: Some("disconnected".into()),
            }),
        }
    }
}

/// Handle a message from the sidecar.
fn handle_sidecar_message(
    msg: SidecarMessage,
    accounts: AccountStateMap,
    event_sink: Option<Arc<dyn ChannelEventSink>>,
    _message_log: Option<Arc<dyn MessageLog>>,
) {
    match msg {
        SidecarMessage::Qr { account_id, qr } => {
            debug!(account_id, "received QR code from sidecar");
            let mut accounts = accounts.write().unwrap();
            if let Some(state) = accounts.get_mut(&account_id) {
                state.connection_state = ConnectionState::QrReceived(qr);
            }
        },
        SidecarMessage::Connected {
            account_id,
            phone_number,
        } => {
            info!(account_id, ?phone_number, "whatsapp web connected");
            let mut accounts = accounts.write().unwrap();
            if let Some(state) = accounts.get_mut(&account_id) {
                state.connection_state = ConnectionState::Connected { phone_number };
            }
        },
        SidecarMessage::Disconnected { account_id, reason } => {
            warn!(account_id, reason, "whatsapp web disconnected");
            let mut accounts = accounts.write().unwrap();
            if let Some(state) = accounts.get_mut(&account_id) {
                state.connection_state = ConnectionState::Disconnected;
            }
        },
        SidecarMessage::LoggedOut { account_id } => {
            info!(account_id, "whatsapp web logged out");
            let mut accounts = accounts.write().unwrap();
            if let Some(state) = accounts.get_mut(&account_id) {
                state.connection_state = ConnectionState::Disconnected;
            }
        },
        SidecarMessage::InboundMessage {
            account_id,
            message_id: _,
            chat_jid,
            sender_jid,
            sender_name,
            is_group: _,
            body,
            media_type: _,
            media_url: _,
            quoted_message_id: _,
            quoted_body: _,
            timestamp: _,
        } => {
            debug!(account_id, sender_jid, "received inbound message");

            // Get config to check allowlist.
            let (config, sink) = {
                let accounts = accounts.read().unwrap();
                accounts.get(&account_id).map_or((None, None), |s| {
                    (Some(s.config.clone()), s.event_sink.clone())
                })
            };

            let sink = sink.or(event_sink);

            // Check allowlist.
            let access_granted = config
                .as_ref()
                .map(|c| {
                    c.allowlist.is_empty() || c.allowlist.iter().any(|p| sender_jid.contains(p))
                })
                .unwrap_or(true);

            // Emit event for UI.
            if let Some(sink) = sink.clone() {
                let event = ChannelEvent::InboundMessage {
                    channel_type: "whatsapp-web".to_string(),
                    account_id: account_id.clone(),
                    peer_id: sender_jid.clone(),
                    username: sender_name.clone(),
                    sender_name: sender_name.clone(),
                    message_count: None,
                    access_granted,
                };
                tokio::spawn(async move {
                    sink.emit(event).await;
                });
            }

            // Dispatch to chat if access granted.
            if access_granted
                && let Some(sink) = sink
            {
                let reply_target = ChannelReplyTarget {
                    channel_type: "whatsapp-web".to_string(),
                    account_id: account_id.clone(),
                    chat_id: chat_jid,
                };
                let meta = ChannelMessageMeta {
                    channel_type: "whatsapp-web".to_string(),
                    sender_name,
                    username: None,
                    model: config.as_ref().and_then(|c| c.model.clone()),
                };
                tokio::spawn(async move {
                    sink.dispatch_to_chat(&body, reply_target, meta).await;
                });
            }
        },
        SidecarMessage::SendResult {
            request_id,
            success,
            message_id: _,
            error,
        } => {
            if success {
                debug!(request_id, "message sent successfully");
            } else {
                warn!(request_id, ?error, "failed to send message");
            }
        },
        SidecarMessage::StatusResponse { .. } => {
            // Status responses are handled separately.
        },
        SidecarMessage::Error { account_id, error } => {
            warn!(?account_id, error, "sidecar error");
        },
    }
}
