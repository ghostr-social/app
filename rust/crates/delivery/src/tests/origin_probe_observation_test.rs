use crate::manager::stats::StatsKeeper;
use crate::manager::transfers::ProbeObservation;
use crate::probe::media::ProbeResult;
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};
use std::time::Duration;

#[tokio::test]
async fn completed_head_probe_updates_the_head_context_only() {
    let root = super::support::temp_directory("ghostr-origin-head");
    let mut keeper = StatsKeeper::load(root.join("stats.json"), Duration::ZERO).await;
    let url = "https://head.example/video.mp4".to_owned();
    keeper.note_probe(&ProbeObservation {
        post: ghostr_engine::PostId::new("clip"),
        url: url.clone(),
        outcome: Ok(ProbeResult {
            content_length: Some(900_000),
            accept_ranges: Some(true),
            content_type: Some("video/mp4".to_owned()),
            validator: None,
            ttfb: Duration::from_millis(35),
        }),
    });
    let now = crate::manager::time::unix_time_ms();
    let query = OriginQuery::new(
        url.clone(),
        OriginContext::new(RequestMethod::Head, 900_000, MediaClass::Unknown)
            .with_network(NetworkClass::Unavailable)
            .with_observed_at_ms(now),
    );

    let estimate = keeper
        .stats()
        .origin_model()
        .estimate(&query, now, DecisionMode::Normal);
    assert!(estimate.effective_samples > 0.0);
    assert!(estimate.ttfb_ms.p50 > 35);
    assert!(estimate.ttfb_ms.p50 < 250);
    assert!(estimate.range_compliance.is_none());

    let range_query = OriginQuery::new(
        url,
        OriginContext::new(RequestMethod::RangeGet, 900_000, MediaClass::Unknown)
            .with_network(NetworkClass::Unavailable)
            .with_observed_at_ms(now),
    );
    let range = keeper
        .stats()
        .origin_model()
        .estimate(&range_query, now, DecisionMode::Normal);
    assert_eq!(range.effective_samples, 0.0);
    assert_eq!(range.ttfb_ms.p50, 300);
    std::fs::remove_dir_all(root).expect("remove fixture");
}
