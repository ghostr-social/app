use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::manager::state::DeliveryState;
use crate::manager::timeline::install_timeline;
use crate::tests::media_timeline_fixture::classic_moov;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn parsed_timeline_is_installed_for_the_matching_representation() {
    let post = PostId::new("post");
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    assert!(state.apply_focus(
        DeliveryFocus::compatibility(
            vec![FocusItem {
                post: post.clone(),
                meta: VideoMeta {
                    urls: vec!["https://media.example/video.mp4".into()],
                    delivery: DeliveryKind::Progressive,
                    sha256: Some("digest".into()),
                    size_bytes: Some(20_000),
                    duration_ms: Some(1_000),
                },
            }],
            0,
            0,
        ),
        0,
    ));
    let binding = state.catalog().binding(&post).unwrap();
    let moov = classic_moov(100, 100);
    let timeline = parse_mp4_segments(&[MediaSegment::new(10_000, &moov)]).unwrap();

    assert!(install_timeline(&mut state, &binding, timeline));
    assert!(state.catalog().lookup(&post).unwrap().timeline().is_some());
}
