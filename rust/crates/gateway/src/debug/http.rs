//! Loopback-only HTTP commands for the progressive delivery lab.

use crate::debug::assets as debug_assets;
use crate::debug::hls as debug_hls;
use crate::debug::state::{self as debug_state, DebugSnapshot};
use crate::debug::videos::{DebugVideoRegistration, DebugVideos};
use crate::hls::sessions::HlsSessions;
use crate::progressive::route::ProgressiveState;
use axum::extract::State;
use axum::http::header::CACHE_CONTROL;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_delivery::delivery_events::DeliveryHandle;
use nostr_sdk::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const MAX_BANDWIDTH_KBPS: u64 = 10_000_000;
const MAX_LATENCY_MS: u64 = 60_000;
const MAX_PACKET_LOSS_BPS: u16 = 10_000;
const MAX_CONNECTIONS: usize = 64;

#[derive(Clone)]
struct DebugHttpState {
    progressive: Arc<ProgressiveState>,
    videos: DebugVideos,
    delivery: DeliveryHandle,
    hls: HlsSessions,
    client: Arc<Client>,
}

#[derive(Deserialize)]
struct AddVideoRequest {
    url: String,
    #[serde(default)]
    mirrors: Vec<String>,
    size_bytes: Option<u64>,
    duration_ms: Option<u64>,
}

#[derive(Serialize)]
struct AddVideoResponse {
    id: String,
}

#[derive(Deserialize)]
struct SelectFocusRequest {
    id: String,
}

#[derive(Deserialize)]
struct StorageBudgetRequest {
    budget_bytes: u64,
}

pub(crate) fn router(
    progressive: Arc<ProgressiveState>,
    delivery: DeliveryHandle,
    hls: HlsSessions,
    client: Arc<Client>,
) -> Router {
    let hls_routes = debug_hls::router(progressive.debug_feed.clone(), hls.clone());
    let state = DebugHttpState {
        progressive,
        videos: DebugVideos::new(delivery.clone()),
        delivery,
        hls,
        client,
    };
    Router::new()
        .route("/debug/api/state", get(current_state))
        .route("/debug/api/network", put(update_network))
        .route("/debug/api/focus", put(select_focus))
        .route("/debug/api/videos", post(add_video))
        .route("/debug/api/storage", put(update_storage))
        .route("/debug/api/data", delete(clear_data))
        .with_state(state)
        .merge(debug_assets::router())
        .merge(hls_routes)
}

async fn current_state(State(state): State<DebugHttpState>) -> impl IntoResponse {
    let snapshot: DebugSnapshot = debug_state::snapshot(&state.progressive, &state.delivery).await;
    ([(CACHE_CONTROL, "no-store")], Json(snapshot))
}

async fn update_network(
    State(state): State<DebugHttpState>,
    Json(profile): Json<NetworkProfile>,
) -> Result<Json<NetworkProfile>, StatusCode> {
    validate_network(profile)?;
    state.progressive.network.update(profile);
    state.delivery.update_network_profile(profile);
    Ok(Json(profile))
}

async fn select_focus(
    State(state): State<DebugHttpState>,
    Json(request): Json<SelectFocusRequest>,
) -> Result<StatusCode, StatusCode> {
    let selected_from_feed = state.progressive.debug_feed.select(&request.id).is_ok();
    match selected_from_feed || state.videos.select(&request.id) {
        true => Ok(StatusCode::NO_CONTENT),
        false => Err(StatusCode::NOT_FOUND),
    }
}

async fn add_video(
    State(state): State<DebugHttpState>,
    Json(request): Json<AddVideoRequest>,
) -> Result<(StatusCode, Json<AddVideoResponse>), StatusCode> {
    let registration = DebugVideoRegistration {
        url: request.url,
        mirrors: request.mirrors,
        size_bytes: request.size_bytes,
        duration_ms: request.duration_ms,
    };
    let id = state
        .videos
        .add(registration)
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    Ok((StatusCode::CREATED, Json(AddVideoResponse { id })))
}

async fn update_storage(
    State(state): State<DebugHttpState>,
    Json(request): Json<StorageBudgetRequest>,
) -> Result<StatusCode, StatusCode> {
    state
        .progressive
        .store
        .set_storage_budget(request.budget_bytes)
        .await
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    state.delivery.storage_changed();
    Ok(StatusCode::NO_CONTENT)
}

async fn clear_data(State(state): State<DebugHttpState>) -> Result<StatusCode, StatusCode> {
    state.progressive.debug_feed.clear();
    state.videos.clear();
    let delivery = state.delivery.clear();
    let database = state.client.database().wipe();
    let hls = state.hls.clear();
    let (delivery, database, ()) = tokio::join!(delivery, database, hls);
    delivery.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    database.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

fn validate_network(profile: NetworkProfile) -> Result<(), StatusCode> {
    if profile.bandwidth_kbps > MAX_BANDWIDTH_KBPS
        || profile.latency_ms > MAX_LATENCY_MS
        || profile.packet_loss_bps > MAX_PACKET_LOSS_BPS
        || profile.max_connections_per_host > MAX_CONNECTIONS
    {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }
    Ok(())
}
