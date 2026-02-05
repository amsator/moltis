//! WebSocket communication with the WhatsApp Baileys sidecar.

use std::sync::Arc;

use {
    anyhow::{Context, Result},
    futures::{SinkExt, StreamExt},
    tokio::sync::{RwLock, mpsc, oneshot},
    tokio_tungstenite::{connect_async, tungstenite::Message},
    tracing::{debug, error, info, warn},
};

use crate::types::{GatewayMessage, SidecarMessage};

/// Default sidecar WebSocket port.
pub const DEFAULT_SIDECAR_PORT: u16 = 9876;

/// Handle for communicating with the sidecar.
#[derive(Clone)]
pub struct SidecarHandle {
    /// Sender for outgoing messages to the sidecar.
    tx: mpsc::Sender<GatewayMessage>,
    /// Connection state.
    connected: Arc<RwLock<bool>>,
}

impl SidecarHandle {
    /// Send a message to the sidecar.
    pub async fn send(&self, msg: GatewayMessage) -> Result<()> {
        self.tx
            .send(msg)
            .await
            .context("failed to send message to sidecar")
    }

    /// Check if connected to the sidecar.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
}

/// Callback for handling messages from the sidecar.
pub type MessageCallback = Arc<dyn Fn(SidecarMessage) + Send + Sync>;

/// Connect to the sidecar and spawn reader/writer tasks.
pub async fn connect_to_sidecar(
    port: u16,
    on_message: MessageCallback,
) -> Result<(SidecarHandle, oneshot::Receiver<()>)> {
    let url = format!("ws://127.0.0.1:{port}");
    info!(url = %url, "connecting to WhatsApp sidecar");

    let (ws_stream, _) = connect_async(&url)
        .await
        .context("failed to connect to sidecar")?;

    info!("connected to WhatsApp sidecar");

    let (mut write, mut read) = ws_stream.split();

    // Channel for outgoing messages.
    let (tx, mut rx) = mpsc::channel::<GatewayMessage>(32);

    // Channel for disconnect notification.
    let (disconnect_tx, disconnect_rx) = oneshot::channel();

    let connected = Arc::new(RwLock::new(true));
    let connected_reader = Arc::clone(&connected);
    let connected_writer = Arc::clone(&connected);

    // Spawn reader task.
    tokio::spawn(async move {
        while let Some(msg_result) = read.next().await {
            match msg_result {
                Ok(Message::Text(text)) => match serde_json::from_str::<SidecarMessage>(&text) {
                    Ok(msg) => {
                        debug!(?msg, "received message from sidecar");
                        on_message(msg);
                    },
                    Err(e) => {
                        warn!(error = %e, text = %text, "failed to parse sidecar message");
                    },
                },
                Ok(Message::Close(_)) => {
                    info!("sidecar connection closed");
                    break;
                },
                Ok(_) => {}, // Ignore ping/pong/binary
                Err(e) => {
                    error!(error = %e, "WebSocket read error");
                    break;
                },
            }
        }

        *connected_reader.write().await = false;
        let _ = disconnect_tx.send(());
    });

    // Spawn writer task.
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if let Err(e) = write.send(Message::Text(json.into())).await {
                        error!(error = %e, "failed to send message to sidecar");
                        break;
                    }
                    debug!(?msg, "sent message to sidecar");
                },
                Err(e) => {
                    error!(error = %e, "failed to serialize message");
                },
            }
        }

        *connected_writer.write().await = false;
    });

    Ok((SidecarHandle { tx, connected }, disconnect_rx))
}

/// Try to connect to the sidecar with retries.
pub async fn connect_with_retry(
    port: u16,
    on_message: MessageCallback,
    max_retries: u32,
) -> Result<(SidecarHandle, oneshot::Receiver<()>)> {
    let mut attempt = 0;
    loop {
        match connect_to_sidecar(port, Arc::clone(&on_message)).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt >= max_retries {
                    return Err(e);
                }
                warn!(
                    attempt,
                    max_retries,
                    error = %e,
                    "failed to connect to sidecar, retrying..."
                );
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            },
        }
    }
}
