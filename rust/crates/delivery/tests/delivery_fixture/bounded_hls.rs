use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Response, Uri};
use axum::routing::get;
use axum::Router;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

pub const INIT_BYTES: usize = 700 * 1024;
pub type Requests = Arc<Mutex<Vec<(String, Option<String>, Option<String>)>>>;

pub async fn serve() -> (String, Requests) {
    let requests = Requests::default();
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let app = Router::new()
        .fallback(get(object))
        .with_state(std::sync::Arc::clone(&requests));
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("valid test fixture");
    });
    (format!("http://{address}/index.m3u8"), requests)
}

async fn object(State(requests): State<Requests>, uri: Uri, headers: HeaderMap) -> Response<Body> {
    match uri.path() {
        "/index.m3u8" => full(
            b"#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
            "root-v1",
        ),
        "/init.mp4" => partial(&headers, &requests),
        "/segment.m4s" => full(b"segment", "segment-v1"),
        _ => Response::builder()
            .status(404)
            .body(Body::empty())
            .expect("valid test fixture"),
    }
}

fn partial(headers: &HeaderMap, requests: &Requests) -> Response<Body> {
    let requested = headers
        .get(header::RANGE)
        .expect("valid test fixture")
        .to_str()
        .expect("valid test fixture");
    let if_range = headers
        .get(header::IF_RANGE)
        .map(|value| value.to_str().expect("valid test fixture").to_owned());
    let if_match = headers
        .get(header::IF_MATCH)
        .map(|value| value.to_str().expect("valid test fixture").to_owned());
    requests
        .lock()
        .expect("valid test fixture")
        .push((requested.to_owned(), if_range, if_match));
    let (start, end) = requested
        .strip_prefix("bytes=")
        .expect("valid test fixture")
        .split_once('-')
        .expect("valid test fixture");
    ranged(
        start.parse().expect("valid test fixture"),
        end.parse().expect("valid test fixture"),
    )
}

fn ranged(start: usize, requested_end: usize) -> Response<Body> {
    let end = requested_end.min(INIT_BYTES - 1);
    Response::builder()
        .status(206)
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{INIT_BYTES}"),
        )
        .header(header::CONTENT_LENGTH, end - start + 1)
        .header(header::ETAG, "\"init-v1\"")
        .body(Body::from(init_range(start, end)))
        .expect("valid test fixture")
}

pub fn init_body() -> Vec<u8> {
    init_range(0, INIT_BYTES - 1)
}

fn init_range(start: usize, end: usize) -> Vec<u8> {
    (start..=end).map(|index| (index % 251) as u8).collect()
}

fn full(body: &'static [u8], etag: &'static str) -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::ETAG, format!("\"{etag}\""))
        .body(Body::from(body))
        .expect("valid test fixture")
}
