use super::super::axiom_test_support::{fetch, FetchInput};
use super::super::FetchSpec;
use super::support::{client, network_status};
use ghostr_engine::adaptive::{PreemptionAuthority, REQUEST_SLICE_BYTES};
use ghostr_engine::origin_model::ErrorReason;
use ghostr_hls_manifest::hls_manifest::MAX_HLS_ASSET_BYTES;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;

#[tokio::test]
async fn non_final_partial_response_must_fill_the_authorized_block() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("valid test fixture");
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.expect("valid test fixture") > 0);
        socket
            .write_all(response().as_bytes())
            .await
            .expect("valid test fixture");
    });
    let url = format!("http://{address}/init.mp4");
    let requests = client();
    let network = network_status();

    let Err(failure) = fetch(
        &requests,
        FetchInput {
            spec: FetchSpec {
                url: &url,
                limit: REQUEST_SLICE_BYTES as usize,
                object_limit: MAX_HLS_ASSET_BYTES as u64,
                object: Default::default(),
                timeouts: HlsTransferTimeouts::default(),
                priority: PreemptionAuthority::Transition,
                admission_fence: None,
            },
            traffic: None,
        },
        &network,
        None,
    )
    .await
    else {
        panic!("short non-final range must fail")
    };

    assert_eq!(failure.reason(), ErrorReason::RangeNoncompliant);
    server.await.expect("valid test fixture");
}

fn response() -> &'static str {
    "HTTP/1.1 206 Partial Content\r\nContent-Range: bytes 0-65535/307200\r\nContent-Length: 65536\r\nETag: \"v1\"\r\n\r\n"
}
