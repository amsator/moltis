//! Main WASM client for browser integration.

use {serde::de::Error as _, wasm_bindgen::prelude::*};

use crate::{config::WasmConfig, error::WasmError, storage::MemoryStorage};

/// The main moltis client for browser environments.
///
/// This client provides a JavaScript-friendly interface for interacting
/// with moltis. It can either:
/// 1. Connect to a backend gateway for full functionality
/// 2. Run in standalone mode with limited features (no tool execution)
#[wasm_bindgen]
pub struct MoltisClient {
    config: WasmConfig,
    #[allow(dead_code)]
    storage: MemoryStorage,
    #[allow(dead_code)]
    session_id: Option<String>,
}

#[wasm_bindgen]
impl MoltisClient {
    /// Create a new moltis client with default configuration.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            config: WasmConfig::default(),
            storage: MemoryStorage::new(),
            session_id: None,
        }
    }

    /// Create a new client with the given configuration.
    #[wasm_bindgen(js_name = withConfig)]
    pub fn with_config(config: WasmConfig) -> Self {
        Self {
            config,
            storage: MemoryStorage::new(),
            session_id: None,
        }
    }

    /// Set the backend gateway URL.
    #[wasm_bindgen(js_name = setBackendUrl)]
    pub fn set_backend_url(&mut self, url: String) {
        self.config.set_backend_url(url);
    }

    /// Get the current backend URL.
    #[wasm_bindgen(js_name = getBackendUrl)]
    pub fn get_backend_url(&self) -> Option<String> {
        self.config.get_backend_url()
    }

    /// Check if the client is connected to a backend.
    #[wasm_bindgen(js_name = hasBackend)]
    pub fn has_backend(&self) -> bool {
        self.config.backend_url.is_some()
    }

    /// Get the current session ID.
    #[wasm_bindgen(js_name = getSessionId)]
    pub fn get_session_id(&self) -> Option<String> {
        self.session_id.clone()
    }

    /// Start a new session.
    #[wasm_bindgen(js_name = newSession)]
    pub fn new_session(&mut self) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.session_id = Some(session_id.clone());
        tracing::info!(session_id = %session_id, "started new session");
        session_id
    }

    /// Parse a protocol frame from JSON.
    #[wasm_bindgen(js_name = parseFrame)]
    pub fn parse_frame(&self, json: &str) -> Result<JsValue, WasmError> {
        let frame: moltis_protocol::GatewayFrame =
            serde_json::from_str(json).map_err(WasmError::Serialization)?;
        serde_wasm_bindgen::to_value(&frame)
            .map_err(|e| WasmError::Serialization(serde_json::Error::custom(e.to_string())))
    }

    /// Serialize a protocol frame to JSON.
    #[wasm_bindgen(js_name = serializeFrame)]
    pub fn serialize_frame(&self, frame: JsValue) -> Result<String, WasmError> {
        let frame: moltis_protocol::GatewayFrame = serde_wasm_bindgen::from_value(frame)
            .map_err(|e| WasmError::Serialization(serde_json::Error::custom(e.to_string())))?;
        serde_json::to_string(&frame).map_err(WasmError::Serialization)
    }
}

impl Default for MoltisClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = MoltisClient::new();
        assert!(client.get_backend_url().is_none());
        assert!(!client.has_backend());
    }

    #[test]
    fn test_client_with_backend() {
        let mut client = MoltisClient::new();
        client.set_backend_url("http://localhost:3000".to_string());
        assert_eq!(
            client.get_backend_url(),
            Some("http://localhost:3000".to_string())
        );
        assert!(client.has_backend());
    }

    #[test]
    fn test_new_session() {
        let mut client = MoltisClient::new();
        assert!(client.get_session_id().is_none());

        let session_id = client.new_session();
        assert!(!session_id.is_empty());
        assert_eq!(client.get_session_id(), Some(session_id));
    }
}
