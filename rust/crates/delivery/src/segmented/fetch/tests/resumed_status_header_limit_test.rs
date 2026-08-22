use super::super::{fetch, FetchInput, FetchSpec, ObjectRequest};
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
    let failure = match fetch(
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
    {
        Ok(_) => panic!("oversized resumed response headers must fail"),
        Err(failure) => failure,
    };

    assert_eq!(failure.reason(), ErrorReason::InvalidResponse);
    assert!(failure.to_string().contains("headers exceed byte limit"));
    server.await.unwrap();
}
