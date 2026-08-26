use super::response;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(super) struct OriginState {
    gets: Arc<Mutex<Vec<String>>>,
}

impl OriginState {
    pub fn new() -> Self {
        Self {
            gets: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn gets(&self) -> Vec<String> {
        self.gets.lock().expect("valid test fixture").clone()
    }
}

pub(super) fn router(state: OriginState) -> Router {
    Router::new().route("/{id}", get(media)).with_state(state)
}

async fn media(
    Path(id): Path<String>,
    State(state): State<OriginState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if method == Method::HEAD {
        return response::metadata();
    }
    state.gets.lock().expect("valid test fixture").push(id);
    response::partial(&headers)
}
