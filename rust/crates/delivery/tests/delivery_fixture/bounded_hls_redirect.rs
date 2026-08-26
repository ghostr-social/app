use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Response, Uri};
use axum::response::{IntoResponse as _, Redirect};
use axum::routing::get;
use axum::Router;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

pub const INIT_BYTES: usize = 300 * 1024;
pub type Requests = Arc<Mutex<Vec<(String, String)>>>;

#[derive(Clone, Default)]
struct FixtureState {
    logical_requests: Arc<AtomicUsize>,
    requests: Requests,
}

pub async fn serve() -> (String, Requests) {
    let state = FixtureState::default();
    let requests = std::sync::Arc::clone(&state.requests);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let app = Router::new().fallback(get(object)).with_state(state);
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("valid test fixture");
    });
    (format!("http://{address}/index.m3u8"), requests)
}

async fn object(State(state): State<FixtureState>, uri: Uri, headers: HeaderMap) -> Response<Body> {
    match uri.path() {
        "/index.m3u8" => {
            full(b"#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n")
        }
        "/init.mp4" => redirect(&state),
        "/v1/init.mp4" => ranged("v1", &headers, &state),
        "/v2/init.mp4" => ranged("v2", &headers, &state),
        "/segment.m4s" => full(b"segment"),
        _ => Response::builder()
            .status(404)
            .body(Body::empty())
            .expect("valid test fixture"),
    }
}

fn redirect(state: &FixtureState) -> Response<Body> {
    let request = state.logical_requests.fetch_add(1, Ordering::SeqCst);
    let path = if request == 0 {
        "/v1/init.mp4"
    } else {
        "/v2/init.mp4"
    };
    Redirect::temporary(path).into_response()
}

fn ranged(version: &str, headers: &HeaderMap, state: &FixtureState) -> Response<Body> {
    let range = headers
        .get(header::RANGE)
        .expect("valid test fixture")
        .to_str()
        .expect("valid test fixture");
    state
        .requests
        .lock()
        .expect("valid test fixture")
        .push((version.to_owned(), range.to_owned()));
    let (start, requested_end) = parse_range(range);
    let end = requested_end.min(INIT_BYTES - 1);
    let value = if version == "v1" { 7 } else { 8 };
    Response::builder()
        .status(206)
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{INIT_BYTES}"),
        )
        .header(header::CONTENT_LENGTH, end - start + 1)
        .header(header::ETAG, "\"stable\"")
        .body(Body::from(vec![value; end - start + 1]))
        .expect("valid test fixture")
}

fn parse_range(value: &str) -> (usize, usize) {
    let (start, end) = value
        .strip_prefix("bytes=")
        .expect("valid test fixture")
        .split_once('-')
        .expect("valid test fixture");
    (
        start.parse().expect("valid test fixture"),
        end.parse().expect("valid test fixture"),
    )
}

fn full(body: &'static [u8]) -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_LENGTH, body.len())
        .body(Body::from(body))
        .expect("valid test fixture")
}
