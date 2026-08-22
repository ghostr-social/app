use super::requests::RequestLedger;
use super::response;
use axum::extract::State;
use axum::http::{HeaderMap, Method};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub(super) enum HeadBehavior {
    Blocked,
    Rejected,
    Lengthless,
    RangeOpaque,
    RangeBlindSplit,
    DeferredFailure,
}

#[derive(Clone)]
pub(super) struct OriginState {
    pub(super) bytes: Arc<Vec<u8>>,
    pub(super) requests: RequestLedger,
    pub(super) prefix_ready: Arc<tokio::sync::Semaphore>,
    pub(super) release: Arc<tokio::sync::Semaphore>,
    head: HeadBehavior,
}

pub(super) struct RunningOrigin {
    pub(super) url: String,
    pub(super) state: OriginState,
    pub(super) task: tokio::task::JoinHandle<()>,
}

pub(super) async fn start(bytes: Vec<u8>, head: HeadBehavior) -> RunningOrigin {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind progressive origin");
    let address = listener.local_addr().expect("origin address");
    let state = OriginState {
        bytes: Arc::new(bytes),
        requests: RequestLedger::default(),
        prefix_ready: Arc::new(tokio::sync::Semaphore::new(0)),
        release: Arc::new(tokio::sync::Semaphore::new(0)),
        head,
    };
    let app = Router::new()
        .route("/video.mp4", get(serve))
        .with_state(state.clone());
    let task = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("serve progressive origin");
    });
    RunningOrigin {
        url: format!("http://{address}/video.mp4"),
        state,
        task,
    }
}

async fn serve(State(state): State<OriginState>, method: Method, headers: HeaderMap) -> Response {
    state.requests.record(method.clone(), &headers);
    if method == Method::HEAD {
        return match state.head {
            HeadBehavior::Blocked => std::future::pending::<Response>().await,
            HeadBehavior::Rejected => {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                response::rejected_head()
            }
            HeadBehavior::Lengthless => {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                response::lengthless_head()
            }
            HeadBehavior::RangeOpaque => response::range_opaque_head(state.bytes.len()),
            HeadBehavior::RangeBlindSplit => response::range_blind_head(state.bytes.len()),
            HeadBehavior::DeferredFailure => {
                tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                response::lengthless_head()
            }
        };
    }
    if matches!(
        state.head,
        HeadBehavior::Rejected | HeadBehavior::Lengthless
    ) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    if matches!(state.head, HeadBehavior::DeferredFailure) {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        return response::failed_body();
    }
    if matches!(state.head, HeadBehavior::RangeBlindSplit) {
        return response::range_blind_split(state.bytes, state.prefix_ready, state.release);
    }
    response::partial(&state.bytes, &headers)
}
