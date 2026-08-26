use super::super::axiom_test_support::{fetch, FetchInput};
use super::super::{FetchSpec, ObjectRequest};
use super::support::{client, network_status, oversized_status};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::origin_model::ErrorReason;
use ghostr_hls_manifest::hls_manifest::MAX_HLS_ASSET_BYTES;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;

#[tokio::test]
async fn resumed_unsatisfied_range_still_enforces_the_header_limit_first() {
    let (url, server) = oversized_status("416 Range Not Satisfiable").await;
    let requests = client();
    let network = network_status();
    let Err(failure) = fetch(
        &requests,
        FetchInput {
            spec: FetchSpec {
                url: &url,
                limit: 44 * 1024,
                object_limit: MAX_HLS_ASSET_BYTES as u64,
                object: ObjectRequest {
                    offset: 256 * 1024,
                    total: Some(300 * 1024),
                    ..Default::default()
                },
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
        panic!("oversized resumed response headers must fail")
    };

    assert_eq!(failure.reason(), ErrorReason::InvalidResponse);
    assert!(
        format!("{failure:#}").contains("headers exceed byte limit"),
        "{failure:#}"
    );
    server.await.expect("valid test fixture");
}
