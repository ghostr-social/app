//! Fixture endpoints that refuse requests: a HEAD-rejecting range
//! server (probe fallback path) and an always-failing server.

use super::ranged::{app, ranged, serve, Media};
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::Response;
use axum::routing::any;
use axum::Router;

pub async fn serve_head_rejected(bytes: Vec<u8>) -> String {
    serve(app(any(no_head), bytes)).await
}

pub async fn serve_failing() -> String {
    serve(Router::new().route("/video.mp4", any(failing))).await
}

async fn no_head(State(media): State<Media>, method: Method, headers: HeaderMap) -> Response {
    if method == Method::HEAD {
        return Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Body::empty())
            .expect("rejected response");
    }
    ranged(State(media), headers).await
}

async fn failing() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
