#![cfg(all(feature = "device-integration", debug_assertions))]

use ghostr_gateway::device_integration::DeviceIntegrationMediaHttpClient;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn integration_client_reaches_only_its_configured_loopback_origin() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let server = tokio::spawn(serve_once(listener));
    let client = DeviceIntegrationMediaHttpClient::new(&format!("http://{address}"))
        .expect("valid test fixture");

    let response = client
        .get(&format!("http://{address}/video.mp4"))
        .expect("valid test fixture")
        .send()
        .await
        .expect("valid test fixture");

    assert_eq!(response.bytes().await.expect("valid test fixture"), "ok");
    assert!(client.get("http://127.0.0.1:9/private.mp4").is_err());
    assert!(client.get("http://10.0.2.2/private.mp4").is_err());
    server.await.expect("valid test fixture");
}

async fn serve_once(listener: tokio::net::TcpListener) {
    let (mut socket, _) = listener.accept().await.expect("valid test fixture");
    let mut request = [0; 512];
    let read = socket.read(&mut request).await.expect("valid test fixture");
    assert!(read > 0, "client sent an empty request");
    socket
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
        .await
        .expect("valid test fixture");
}
