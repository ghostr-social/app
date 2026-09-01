use crate::delivery_events::{
    DeliveryFocus, FocusItem, PlayerPreparationAttempt, PlayerPreparationAuthority,
    PlayerPreparationObservation, PlayerPreparationReport, PlayerPreparationState,
};
use crate::manager::state::DeliveryState;
use crate::segmented::{HlsPreparedAssetAuthority, SegmentedCache};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

pub(super) struct HlsPreparationFixture {
    pub(super) cache: SegmentedCache,
    pub(super) state: DeliveryState,
    pub(super) segmented: crate::segmented::scheduler::SegmentedDelivery,
    authority: HlsPreparedAssetAuthority,
}

impl HlsPreparationFixture {
    pub(super) fn new() -> Self {
        let cache = SegmentedCache::new();
        let focus = focus();
        let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
        assert!(state.apply_focus(focus.clone(), 1));
        let mut segmented = crate::segmented::scheduler::SegmentedDelivery::new(cache.clone());
        assert!(segmented.apply_focus(&focus));
        publish(&cache);
        let authority = cache.snapshot("stream").authority.expect("HLS authority");
        Self {
            cache,
            state,
            segmented,
            authority,
        }
    }

    pub(super) fn report(&self, state: PlayerPreparationState) -> PlayerPreparationReport {
        let authority = PlayerPreparationAuthority::try_new_hls(
            self.authority.clone(),
            format!("hls-v1:{}", self.authority.asset_revision().value()),
        )
        .expect("HLS player authority");
        PlayerPreparationReport::try_new(
            authority,
            PlayerPreparationAttempt::try_new(1, 1, 1).expect("attempt"),
            1,
            PlayerPreparationObservation::try_new(state, None, 1).expect("observation"),
        )
        .expect("report")
    }
}

fn focus() -> DeliveryFocus {
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new("stream"),
            meta: VideoMeta {
                urls: vec!["https://media.example/index.m3u8".to_owned()],
                delivery: DeliveryKind::Hls,
                sha256: None,
                size_bytes: None,
                duration_ms: Some(4_000),
            },
        }],
        0,
        0,
    )
}

fn publish(cache: &SegmentedCache) {
    let post = PostId::new("stream");
    cache.publish_test_hls(&post, 1, "https://media.example/index.m3u8", b"#EXTM3U\n");
}
