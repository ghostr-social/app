mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::media_client;
use ghostr_gateway::hls_sessions::{HlsResourceId, HlsSessions};
use ghostr_gateway::http_gateway::configured_router_with_hls_client;
use ghostr_media_model::native_models::new_native_downloads;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower::ServiceExt;

#[tokio::test]
async fn resolves_relative_resources_against_the_final_redirect_url() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let address = listener.local_addr().expect("address");
    let upstream = tokio::spawn(serve_redirect(listener, address.port()));
    let sessions = HlsSessions::production();
    let id = sessions
        .acquire(vec![format!("http://{address}/start.m3u8")])
        .await
        .expect("session");
    let app =
        configured_router_with_hls_client(new_native_downloads(), sessions.clone(), media_client());

    let request = Request::builder()
        .uri(format!("/hls/{}/index.m3u8", id.as_str()))
        .body(Body::empty())
        .expect("request");
    let response = app.oneshot(request).await.expect("gateway response");
    let body = to_bytes(response.into_body(), 4096).await.expect("body");
    let body = String::from_utf8(body.to_vec()).expect("manifest");
    let token = body.split("/manifests/").nth(1).expect("token");
    let token = token.split('/').next().expect("token end");
    let token = HlsResourceId::parse(token).expect("resource token");
    let resource = sessions.resource(&id, token).await.expect("resource");

    assert_eq!(resource.url.host_str(), Some("localhost"));
    assert_eq!(resource.url.path(), "/redirected/child.m3u8");
    upstream.await.expect("upstream requests");
}

async fn serve_redirect(listener: TcpListener, port: u16) {
    let redirect = format!(
        "HTTP/1.1 302 Found\r\nLocation: http://localhost:{port}/redirected/master.m3u8\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    serve_once(&listener, redirect.as_bytes()).await;
    let manifest = b"HTTP/1.1 200 OK\r\nContent-Type: application/vnd.apple.mpegurl\r\nContent-Length: 49\r\nConnection: close\r\n\r\n#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1\nchild.m3u8\n";
    serve_once(&listener, manifest).await;
}

async fn serve_once(listener: &TcpListener, response: &[u8]) {
    let (mut socket, _) = listener.accept().await.expect("request");
    let mut request = [0; 2048];
    assert!(socket.read(&mut request).await.expect("read") > 0);
    socket.write_all(response).await.expect("response");
}
