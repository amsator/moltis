//! Configuration for WASM builds.
//!
//! Unlike native builds that read from `~/.moltis/moltis.toml`, WASM builds
//! receive configuration via JavaScript or fetch it from the backend.

use {
    serde::{Deserialize, Serialize},
    wasm_bindgen::prelude::*,
};

/// Configuration for the WASM client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmConfig {
    /// Backend gateway URL for API calls and WebSocket connections.
    #[wasm_bindgen(skip)]
    pub backend_url: Option<String>,

    /// Default model to use for completions.
    #[wasm_bindgen(skip)]
    pub default_model: Option<String>,

    /// Whether to persist sessions to IndexedDB.
    #[wasm_bindgen(skip)]
    pub persist_sessions: bool,
}

#[wasm_bindgen]
impl WasmConfig {
    /// Create a new configuration with defaults.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the backend gateway URL.
    #[wasm_bindgen(js_name = setBackendUrl)]
    pub fn set_backend_url(&mut self, url: String) {
        self.backend_url = Some(url);
    }

    /// Get the backend gateway URL.
    #[wasm_bindgen(js_name = getBackendUrl)]
    pub fn get_backend_url(&self) -> Option<String> {
        self.backend_url.clone()
    }

    /// Set the default model.
    #[wasm_bindgen(js_name = setDefaultModel)]
    pub fn set_default_model(&mut self, model: String) {
        self.default_model = Some(model);
    }

    /// Get the default model.
    #[wasm_bindgen(js_name = getDefaultModel)]
    pub fn get_default_model(&self) -> Option<String> {
        self.default_model.clone()
    }

    /// Enable or disable session persistence to IndexedDB.
    #[wasm_bindgen(js_name = setPersistSessions)]
    pub fn set_persist_sessions(&mut self, persist: bool) {
        self.persist_sessions = persist;
    }

    /// Check if session persistence is enabled.
    #[wasm_bindgen(js_name = getPersistSessions)]
    pub fn get_persist_sessions(&self) -> bool {
        self.persist_sessions
    }

    /// Create configuration from a JSON string.
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<WasmConfig, JsValue> {
        serde_json::from_str(json).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Serialize configuration to JSON.
    #[wasm_bindgen(js_name = toJson)]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(self).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            backend_url: None,
            default_model: None,
            persist_sessions: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = WasmConfig::new();
        assert!(config.backend_url.is_none());
        assert!(config.default_model.is_none());
        assert!(config.persist_sessions);
    }

    #[test]
    fn test_config_setters() {
        let mut config = WasmConfig::new();
        config.set_backend_url("http://localhost:3000".to_string());
        config.set_default_model("gpt-4".to_string());
        config.set_persist_sessions(false);

        assert_eq!(
            config.get_backend_url(),
            Some("http://localhost:3000".to_string())
        );
        assert_eq!(config.get_default_model(), Some("gpt-4".to_string()));
        assert!(!config.get_persist_sessions());
    }
}
