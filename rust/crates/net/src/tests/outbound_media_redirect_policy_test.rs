use crate::outbound_media_client::public_redirect_policy;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn public_redirect_policy_rejects_a_private_target() {
    let target = TcpListener::bind("127.0.0.1:0").await.expect("target");
    let target_address = target.local_addr().expect("target address");
    let target_server = tokio::spawn(async move {
        let _ = target.accept().await.expect("target request");
    });
    let origin = TcpListener::bind("127.0.0.1:0").await.expect("origin");
    let origin_address = origin.local_addr().expect("origin address");
    let origin_server = tokio::spawn(async move {
        let (mut socket, _) = origin.accept().await.expect("origin request");
        let mut request = [0; 1024];
        let bytes_read = socket.read(&mut request).await.expect("read origin");
        assert!(bytes_read > 0, "origin request was empty");
        let response = format!(
            "HTTP/1.1 302 Found\r\nLocation: http://{target_address}/private\r\nContent-Length: 0\r\n\r\n"
        );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("origin response");
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(public_redirect_policy())
        .build()
        .expect("media client");

    let result = client
        .get(format!("http://{origin_address}/video.mp4"))
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
