//! Error types for WASM builds.

use {thiserror::Error, wasm_bindgen::prelude::*};

/// Errors that can occur in the WASM client.
#[derive(Debug, Error)]
pub enum WasmError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Not initialized: {0}")]
    NotInitialized(String),
}

impl From<WasmError> for JsValue {
    fn from(err: WasmError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}

impl From<anyhow::Error> for WasmError {
    fn from(err: anyhow::Error) -> Self {
        WasmError::Backend(err.to_string())
    }
}
