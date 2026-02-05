//! WhatsApp Web channel plugin for moltis.
//!
//! Implements `ChannelPlugin` using the WhatsApp Web protocol via Baileys
//! (a Node.js sidecar process) to receive and send messages.

pub mod config;
pub mod outbound;
pub mod plugin;
pub mod sidecar;
pub mod state;
pub mod types;

pub use {config::WhatsAppConfig, plugin::WhatsAppPlugin, sidecar::DEFAULT_SIDECAR_PORT};
