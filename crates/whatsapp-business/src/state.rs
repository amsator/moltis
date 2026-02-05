//! WhatsApp account state management.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use moltis_channels::{ChannelEventSink, message_log::MessageLog};

use crate::{config::WhatsAppAccountConfig, outbound::WhatsAppOutbound};

/// Shared account state map.
pub type AccountStateMap = Arc<RwLock<HashMap<String, AccountState>>>;

/// Per-account runtime state.
pub struct AccountState {
    pub account_id: String,
    pub config: WhatsAppAccountConfig,
    pub outbound: Arc<WhatsAppOutbound>,
    pub message_log: Option<Arc<dyn MessageLog>>,
    pub event_sink: Option<Arc<dyn ChannelEventSink>>,
    /// HTTP client for API calls.
    pub http_client: reqwest::Client,
}
