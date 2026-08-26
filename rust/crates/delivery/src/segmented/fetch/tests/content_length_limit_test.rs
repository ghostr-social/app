use super::asset;
use super::support::client;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_hls_manifest::hls_manifest::MAX_HLS_ASSET_BYTES;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;

#[tokio::test]
async fn declared_hls_body_above_the_authorized_limit_is_rejected_before_reading() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("valid test fixture");
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.expect("valid test fixture") > 0);
        let length = MAX_HLS_ASSET_BYTES + 1;
        let headers = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n");
        socket
            .write_all(headers.as_bytes())
            .await
            .expect("valid test fixture");
    });
    let url =
        url::Url::parse(&format!("http://{address}/segment.m4s")).expect("valid test fixture");

    let Err(error) = asset(&client(), &url, PreemptionAuthority::Transition).await else {
        panic!("oversized declared body must fail before body IO")
    };

    assert!(error
        .to_string()
        .contains("full HLS object exceeds its byte grant"));
    server.await.expect("valid test fixture");
}
