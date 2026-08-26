mod redirect_gate_fixture;

use core::time::Duration;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::response_limits::MAX_MEDIA_RESPONSE_HEADER_BYTES;
use redirect_gate_fixture::OneHopClient;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[tokio::test]
async fn oversized_intermediate_headers_stop_before_the_redirect_target() {
    let target = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let target_url = format!(
        "http://{}/asset",
        target.local_addr().expect("valid test fixture")
    );
    let (hit, mut hit_event) = oneshot::channel();
    tokio::spawn(async move {
        let (mut socket, _) = target.accept().await.expect("valid test fixture");
        let _ = hit.send(());
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
            .expect("valid test fixture");
    });
    let origin = oversized_redirect(target_url).await;
    let executor = MediaRequestExecutor::new(
        OneHopClient::shared(),
        MediaRequestLimits::try_new(1, 1).expect("valid test fixture"),
    );

    let result = executor
        .get(&origin, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit()
        .await
        .expect("valid test fixture")
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(30),
        )
        .await;

    let Err(error) = result else {
        panic!("oversized redirect headers must fail")
    };
    assert!(error.to_string().contains("header"));
    let origin_authority = RequestAuthority::from_url(&origin).expect("valid test fixture");
    assert_eq!(executor.active_for(&origin_authority), 0);
    assert!(
        tokio::time::timeout(Duration::from_millis(75), &mut hit_event)
            .await
            .is_err()
    );
}

async fn oversized_redirect(target: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("valid test fixture");
        let mut request = [0; 1024];
        assert!(
            socket.read(&mut request).await.expect("valid test fixture") > 0,
            "redirect server should receive a request"
        );
        let padding = "a".repeat(MAX_MEDIA_RESPONSE_HEADER_BYTES);
        let response = format!(
            "HTTP/1.1 302 Found\r\nLocation: {target}\r\nX-Padding: {padding}\r\nContent-Length: 0\r\n\r\n"
        );
        socket
            .write_all(response.as_bytes())
            .await
            .expect("valid test fixture");
    });
    format!("http://{address}/start")
}
