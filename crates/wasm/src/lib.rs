//! WebAssembly build of moltis for browser environments.
//!
//! This crate provides a WASM-compatible subset of moltis functionality,
//! enabling agent logic to run directly in the browser while delegating
//! I/O-intensive operations (tool execution, file storage) to a backend.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  Browser (WASM)                     │
//! │  - Protocol parsing                 │
//! │  - Message routing                  │
//! │  - Session state (in-memory)        │
//! │  - Provider selection               │
//! └───────────┬─────────────────────────┘
//!             │ HTTP/WebSocket
//! ┌───────────▼─────────────────────────┐
//! │  Backend Gateway                    │
//! │  - Tool execution                   │
//! │  - File storage                     │
//! │  - LLM API calls (optional)         │
//! └─────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```javascript
//! import init, { MoltisClient } from 'moltis-wasm';
//!
//! await init();
//! const client = new MoltisClient();
//! client.set_backend_url("https://your-backend.example.com");
//!
//! // Send a message and get streaming response
//! const stream = client.send_message("Hello, world!");
//! for await (const chunk of stream) {
//!     console.log(chunk);
//! }
//! ```

use wasm_bindgen::prelude::*;

pub mod client;
pub mod config;
pub mod error;
pub mod storage;

pub use {client::MoltisClient, config::WasmConfig, error::WasmError};

/// Initialize the WASM module.
///
/// This sets up panic hooks for better error messages and initializes
/// the tracing subscriber for logging.
#[wasm_bindgen(start)]
pub fn init() {
    // Set up panic hook for better error messages in browser console
    #[cfg(feature = "console-panic")]
    console_error_panic_hook::set_once();

    // Initialize tracing for WASM
    tracing_wasm::set_as_global_default();

    tracing::info!("moltis-wasm initialized");
}

/// Returns the version of the moltis-wasm crate.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
