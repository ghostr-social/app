mod redirect_gate_fixture;

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::response_limits::MAX_MEDIA_RESPONSE_HEADER_BYTES;
use redirect_gate_fixture::OneHopClient;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[tokio::test]
async fn oversized_intermediate_headers_stop_before_the_redirect_target() {
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_url = format!("http://{}/asset", target.local_addr().unwrap());
    let (hit, mut hit_event) = oneshot::channel();
    tokio::spawn(async move {
        let (mut socket, _) = target.accept().await.unwrap();
        let _ = hit.send(());
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
    });
    let origin = oversized_redirect(target_url).await;
    let executor = MediaRequestExecutor::new(
        OneHopClient::shared(),
        MediaRequestLimits::try_new(1, 1).unwrap(),
    );

    let result = executor
        .get(&origin, PreemptionAuthority::Transition)
        .unwrap()
        .admit()
        .await
        .unwrap()
        .send()
        .await;

    let error = match result {
        Ok(_) => panic!("oversized redirect headers must fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("header"));
    assert!(executor.active_connections().is_empty());
    assert!(
        tokio::time::timeout(Duration::from_millis(75), &mut hit_event)
            .await
            .is_err()
    );
}

async fn oversized_redirect(target: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.unwrap() > 0);
        let padding = "a".repeat(MAX_MEDIA_RESPONSE_HEADER_BYTES);
        let response = format!(
            "HTTP/1.1 302 Found\r\nLocation: {target}\r\nX-Padding: {padding}\r\nContent-Length: 0\r\n\r\n"
        );
        socket.write_all(response.as_bytes()).await.unwrap();
    });
    format!("http://{address}/start")
}
