//! WhatsApp channel plugin for moltis.
//!
//! Implements `ChannelPlugin` using the WhatsApp Cloud API to receive and send
//! messages via webhooks.

pub mod access;
pub mod config;
pub mod outbound;
pub mod plugin;
pub mod state;
pub mod types;
pub mod webhook;

pub use {config::WhatsAppAccountConfig, plugin::WhatsAppPlugin};
