//! WhatsApp webhook routes for the gateway.

use std::sync::Arc;

use {
    axum::{
        body::Bytes,
        extract::{Path, Query, State},
        http::{HeaderMap, StatusCode},
        response::IntoResponse,
    },
    serde::Deserialize,
    tokio::sync::RwLock,
    tracing::{debug, warn},
};

use moltis_whatsapp_business::{
    WhatsAppPlugin,
    types::WebhookPayload,
    webhook::{process_webhook, verify_signature, verify_webhook_subscription},
};

/// Query parameters for webhook verification.
#[derive(Debug, Deserialize)]
pub struct VerifyQuery {
    #[serde(rename = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
    #[serde(rename = "hub.challenge")]
    pub hub_challenge: Option<String>,
}

/// State shared with WhatsApp webhook handlers.
pub struct WhatsAppWebhookState {
    pub plugin: Arc<RwLock<WhatsAppPlugin>>,
}

/// GET handler for webhook verification.
///
/// WhatsApp sends a GET request with:
/// - `hub.mode=subscribe`
/// - `hub.verify_token=<your_verify_token>`
/// - `hub.challenge=<random_string>`
///
/// We respond with the challenge if verification succeeds.
pub async fn whatsapp_webhook_verify(
    State(state): State<Arc<WhatsAppWebhookState>>,
    Path(account_id): Path<String>,
    Query(params): Query<VerifyQuery>,
) -> impl IntoResponse {
    debug!(account_id, "WhatsApp webhook verification request");

    let config = {
        let plugin = state.plugin.read().await;
        plugin.get_account_config(&account_id)
    };

    let Some(config) = config else {
        warn!(account_id, "WhatsApp account not found for verification");
        return (StatusCode::NOT_FOUND, "Account not found".to_string());
    };

    match verify_webhook_subscription(
        params.hub_mode.as_deref(),
        params.hub_verify_token.as_deref(),
        params.hub_challenge.as_deref(),
        &config,
    ) {
        Some(challenge) => {
            debug!(account_id, "WhatsApp webhook verified successfully");
            (StatusCode::OK, challenge)
        },
        None => {
            warn!(account_id, "WhatsApp webhook verification failed");
            (StatusCode::FORBIDDEN, "Verification failed".to_string())
        },
    }
}

/// POST handler for inbound webhooks.
///
/// WhatsApp sends POST requests with:
/// - `X-Hub-Signature-256: sha256=<signature>` header
/// - JSON body with the webhook payload
pub async fn whatsapp_webhook_post(
    State(state): State<Arc<WhatsAppWebhookState>>,
    Path(account_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    debug!(account_id, "WhatsApp webhook POST received");

    // Get the signature header.
    let signature = match headers.get("x-hub-signature-256") {
        Some(v) => match v.to_str() {
            Ok(s) => s,
            Err(_) => {
                warn!(account_id, "Invalid X-Hub-Signature-256 header encoding");
                return StatusCode::BAD_REQUEST;
            },
        },
        None => {
            warn!(account_id, "Missing X-Hub-Signature-256 header");
            return StatusCode::UNAUTHORIZED;
        },
    };

    // Get the account config to verify the signature.
    let (config, accounts) = {
        let plugin = state.plugin.read().await;
        match plugin.get_account_config(&account_id) {
            Some(cfg) => (cfg, plugin.accounts()),
            None => {
                warn!(account_id, "WhatsApp account not found");
                return StatusCode::NOT_FOUND;
            },
        }
    };

    // Verify the signature.
    use secrecy::ExposeSecret;
    if !verify_signature(&body, signature, config.app_secret.expose_secret()) {
        warn!(account_id, "WhatsApp webhook signature verification failed");
        return StatusCode::UNAUTHORIZED;
    }

    // Parse the payload.
    let payload: WebhookPayload = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            warn!(account_id, error = %e, "Failed to parse WhatsApp webhook payload");
            // Return 200 OK even on parse errors to prevent retries.
            return StatusCode::OK;
        },
    };

    // Process the webhook asynchronously.
    // We spawn a task so we can return 200 OK immediately.
    let account_id_clone = account_id.clone();
    tokio::spawn(async move {
        process_webhook(&account_id_clone, &accounts, payload).await;
    });

    // Always return 200 OK to acknowledge receipt.
    StatusCode::OK
}
