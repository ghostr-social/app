use crate::outbound_media_client::public_redirect_policy;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn public_redirect_policy_rejects_credentials_before_following() {
    let origin = TcpListener::bind("127.0.0.1:0").await.expect("origin");
    let address = origin.local_addr().expect("origin address");
    let server = tokio::spawn(async move {
        let (mut socket, _) = origin.accept().await.expect("origin request");
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.expect("read origin") > 0);
        socket
            .write_all(
                b"HTTP/1.1 302 Found\r\nLocation: https://user:secret@93.184.216.34/media.mp4\r\nContent-Length: 0\r\n\r\n",
            )
            .await
            .expect("origin response");
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(public_redirect_policy())
        .build()
        .expect("media client");

    let error = client
        .get(format!("http://{address}/video.mp4"))
        .send()
        .await
        .expect_err("credential redirect must fail");

    server.await.expect("origin server");
    assert!(error.to_string().contains("redirect"));
}
