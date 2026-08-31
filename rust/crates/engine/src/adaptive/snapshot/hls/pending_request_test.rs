use super::*;

#[test]
fn pending_request_query_matches_cursor_transport_and_storage() {
    let start = candidate(HlsObjectCursor::default());
    let live = candidate(HlsObjectCursor::new(
        1,
        0,
        None,
        HlsTransport::ContinueLive {
            response: ActionId::new(9),
        },
    ));
    let tail = candidate(HlsObjectCursor::new(
        2,
        256 * 1024,
        Some(300 * 1024),
        HlsTransport::ResumeRange,
    ));

    assert_eq!(
        start.pending_request_source(256 * 1024, 512 * 1024),
        Some("https://hls.example/root.m3u8")
    );
    assert_eq!(start.pending_request_source(256 * 1024, 1), None);
    assert_eq!(live.pending_request_source(256 * 1024, u64::MAX), None);
    assert_eq!(
        tail.pending_request_source(256 * 1024, 344 * 1024),
        Some("https://hls.example/root.m3u8")
    );
    assert_eq!(
        tail.pending_request_source(256 * 1024, 344 * 1024 - 1),
        None
    );
}

fn candidate(cursor: HlsObjectCursor) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new("current"),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("probability"),
        startup_value_ms: 1_000,
        cursor,
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::RootManifest,
            source: "https://hls.example/root.m3u8".into(),
        },
    }
}
