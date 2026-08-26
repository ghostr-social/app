mod delivery_fixture;
mod hls_terminal_wait;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Response, Uri};
use axum::routing::get;
use axum::Router;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::segmented::SegmentedPhase;
use ghostr_engine::DeliveryKind;
use tokio::net::TcpListener;

#[tokio::test]
async fn hls_signature_accepts_generic_or_missing_manifest_mime() {
    for (label, content_type) in [
        ("generic", Some("application/octet-stream")),
        ("missing", None),
    ] {
        let source = serve(content_type).await;
        let harness = start_harness(label, DeliveryOptions::default());
        let mut item = sized_item("stream", &source, 32, 4_000);
        item.meta.delivery = DeliveryKind::Hls;
        harness.handle.update_focus(focus_now(vec![item], 0, 0));

        let terminal = hls_terminal_wait::wait_terminal(&harness.segmented, "stream").await;

        assert_eq!(terminal.phase, SegmentedPhase::Ready, "{label}");
        harness.handle.clear().await.expect("valid test fixture");
        std::fs::remove_dir_all(&harness.root).ok();
    }
}

async fn serve(content_type: Option<&'static str>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let app = Router::new().fallback(get(object)).with_state(content_type);
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("valid test fixture");
    });
    format!("http://{address}/index.m3u8")
}

async fn object(State(content_type): State<Option<&'static str>>, uri: Uri) -> Response<Body> {
    let body: &'static [u8] = match uri.path() {
        "/index.m3u8" => b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000000\nchild.m3u8\n",
        "/child.m3u8" => b"#EXTM3U\n#EXT-X-TARGETDURATION:4\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
        "/init.mp4" => b"init",
        "/segment.m4s" => b"segment",
        _ => b"missing",
    };
    let mut response = Response::builder();
    if let Some(content_type) = content_type {
        response = response.header(header::CONTENT_TYPE, content_type);
    }
    response.body(Body::from(body)).expect("valid test fixture")
}
