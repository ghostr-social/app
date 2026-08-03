use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn rejects_a_redirect_to_a_private_address() {
    let target = TcpListener::bind("127.0.0.1:0").await.expect("target");
    let target_address = target.local_addr().expect("target address");
    let target_server = tokio::spawn(async move {
        let (mut socket, _) = target.accept().await.expect("target request");
        let mut request = [0; 1024];
        let bytes = socket.read(&mut request).await.expect("read target");
        assert!(bytes > 0, "empty target request");
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
            .expect("target response");
    });
    let origin = TcpListener::bind("127.0.0.1:0").await.expect("origin");
    let origin_address = origin.local_addr().expect("origin address");
    let origin_server = tokio::spawn(async move {
        let (mut socket, _) = origin.accept().await.expect("origin request");
        let mut request = [0; 1024];
        let bytes = socket.read(&mut request).await.expect("read origin");
        assert!(bytes > 0, "empty origin request");
        let response = format!(
            "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/private\r\nContent-Length: 0\r\n\r\n"
        );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("origin response");
    });
    let client = MediaHttpClient::trusted().expect("media client");

    let result = client
        .get(&format!("http://{origin_address}/video.mp4"))
        .expect("request")
        .send()
        .await;

    assert!(result.is_err());
    origin_server.await.expect("origin server");
    assert!(
        !target_server.is_finished(),
        "redirect reached private target"
    );
    target_server.abort();
}
