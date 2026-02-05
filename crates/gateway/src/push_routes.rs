//! Push notification API routes.

use {
    crate::{
        push::{PushPayload, PushService, PushSubscription},
        server::AppState,
    },
    axum::{
        Json, Router,
        extract::State,
        http::StatusCode,
        response::IntoResponse,
        routing::{get, post},
    },
    chrono::Utc,
    serde::{Deserialize, Serialize},
    std::sync::Arc,
};

/// Response with the VAPID public key.
#[derive(Serialize)]
struct VapidKeyResponse {
    public_key: String,
}

/// Request to subscribe to push notifications.
#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: SubscriptionKeys,
}

#[derive(Deserialize)]
pub struct SubscriptionKeys {
    pub p256dh: String,
    pub auth: String,
}

/// Request to unsubscribe from push notifications.
#[derive(Deserialize)]
pub struct UnsubscribeRequest {
    pub endpoint: String,
}

/// Status response.
#[derive(Serialize)]
struct PushStatusResponse {
    enabled: bool,
    subscription_count: usize,
}

/// Get the VAPID public key for push subscription.
async fn vapid_key_handler(
    State(state): State<AppState>,
) -> Result<Json<VapidKeyResponse>, StatusCode> {
    let Some(ref push_service) = state.push_service else {
        return Err(StatusCode::NOT_IMPLEMENTED);
    };

    let public_key = push_service
        .vapid_public_key()
        .await
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(VapidKeyResponse { public_key }))
}

/// Subscribe to push notifications.
async fn subscribe_handler(
    State(state): State<AppState>,
    Json(req): Json<SubscribeRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let Some(ref push_service) = state.push_service else {
        return Err(StatusCode::NOT_IMPLEMENTED);
    };

    let subscription = PushSubscription {
        endpoint: req.endpoint,
        p256dh: req.keys.p256dh,
        auth: req.keys.auth,
        user_agent: None,
        created_at: Utc::now(),
    };

    push_service
        .add_subscription(subscription)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

/// Unsubscribe from push notifications.
async fn unsubscribe_handler(
    State(state): State<AppState>,
    Json(req): Json<UnsubscribeRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let Some(ref push_service) = state.push_service else {
        return Err(StatusCode::NOT_IMPLEMENTED);
    };

    push_service
        .remove_subscription(&req.endpoint)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Get push notification status.
async fn status_handler(State(state): State<AppState>) -> Json<PushStatusResponse> {
    let (enabled, subscription_count) = if let Some(ref push_service) = state.push_service {
        (true, push_service.subscription_count().await)
    } else {
        (false, 0)
    };

    Json(PushStatusResponse {
        enabled,
        subscription_count,
    })
}

/// Create the push notification router.
pub fn push_router() -> Router<AppState> {
    Router::new()
        .route("/vapid-key", get(vapid_key_handler))
        .route("/subscribe", post(subscribe_handler))
        .route("/unsubscribe", post(unsubscribe_handler))
        .route("/status", get(status_handler))
}

/// Send a push notification to all subscribers.
pub async fn send_push_notification(
    push_service: &Arc<PushService>,
    title: &str,
    body: &str,
    url: Option<&str>,
    session_key: Option<&str>,
) -> anyhow::Result<usize> {
    let payload = PushPayload {
        title: title.to_string(),
        body: body.to_string(),
        url: url.map(String::from),
        session_key: session_key.map(String::from),
    };

    push_service.send_to_all(&payload).await
}
