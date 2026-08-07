//! Embedded static assets for the loopback debugger.

use axum::http::header::CONTENT_TYPE;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;

const INDEX: &str = include_str!("debug_assets/index.html");
const APP: &str = include_str!("debug_assets/app.js");
const PLAYER_EVENTS: &str = include_str!("debug_assets/player_events.js");
const NAVIGATION: &str = include_str!("debug_assets/navigation.js");
const NETWORK_MODAL: &str = include_str!("debug_assets/network_modal.js");
const NOSTR_FEED: &str = include_str!("debug_assets/nostr_feed.js");
const VIDEO_FORM: &str = include_str!("debug_assets/video_form.js");
const CLEAR_DATA: &str = include_str!("debug_assets/clear_data.js");
const HLS_SCRIPT: &str = include_str!("debug_assets/hls.min.js");
const HLS_PLAYER: &str = include_str!("debug_assets/hls_player.js");
const HLS_LICENSE: &str = include_str!("debug_assets/hls.LICENSE.txt");
const STYLES: &str = include_str!("debug_assets/styles.css");

pub(crate) fn router() -> Router {
    Router::new()
        .route("/debug", get(index))
        .route("/debug/", get(index))
        .route("/debug/app.js", get(|| script(APP)))
        .route("/debug/player_events.js", get(|| script(PLAYER_EVENTS)))
        .route("/debug/navigation.js", get(|| script(NAVIGATION)))
        .route("/debug/network_modal.js", get(|| script(NETWORK_MODAL)))
        .route("/debug/nostr_feed.js", get(|| script(NOSTR_FEED)))
        .route("/debug/video_form.js", get(|| script(VIDEO_FORM)))
        .route("/debug/clear_data.js", get(|| script(CLEAR_DATA)))
        .route("/debug/hls.min.js", get(|| script(HLS_SCRIPT)))
        .route("/debug/hls_player.js", get(|| script(HLS_PLAYER)))
        .route("/debug/hls.LICENSE.txt", get(hls_license))
        .route("/debug/styles.css", get(styles))
}

async fn index() -> Html<&'static str> {
    Html(INDEX)
}

async fn script(body: &'static str) -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/javascript; charset=utf-8")], body)
}

async fn hls_license() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], HLS_LICENSE)
}

async fn styles() -> impl IntoResponse {
    ([(CONTENT_TYPE, "text/css; charset=utf-8")], STYLES)
}
