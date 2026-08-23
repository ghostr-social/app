use super::startup::startup_certificates;
use super::state::DeliveryState;
use crate::delivery_events::{DeliveryFocus, FocusItem};
use crate::tests::support::temp_directory;
use ghostr_engine::adaptive::{
    candidate_snapshot, AllocationPlan, CandidateEvidence, FeedOffset, ReadyReserveEvidence,
    ReserveCandidateEvidence, ReserveCandidateState, ViewProbability,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::media_timeline::StartupFootprint;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ByteRange, DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::{PartialRangeStore, StoredMediaSnapshot};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn certificate_stays_bound_to_the_planning_snapshot() {
    let post = PostId::new("next");
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility(
            vec![FocusItem {
                post: post.clone(),
                meta: meta("planned"),
            }],
            0,
            0,
        ),
        1,
    );
    let planned = snapshot(state.catalog().binding(&post).unwrap(), "planned").await;
    let mut replacement_catalog = Catalog::new();
    let replacement = snapshot(
        replacement_catalog.upsert(post.clone(), meta("replacement")),
        "replacement",
    )
    .await;
    let startup = complete_startup(state.catalog(), post.clone());
    let plan = AllocationPlan {
        ready_reserve: ReadyReserveEvidence {
            candidates: vec![ReserveCandidateEvidence {
                post: post.clone(),
                state: ReserveCandidateState::Ready { startup },
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let snapshots = HashMap::from([(post, planned.clone())]);
    let certificates = startup_certificates(&state, &plan, &snapshots);
    assert_eq!(certificates.len(), 1);
    assert!(certificates[0].still_valid_in(&planned));
    assert!(!certificates[0].still_valid_in(&replacement));
}
fn complete_startup(catalog: &Catalog, post: PostId) -> StartupFootprint {
    candidate_snapshot(
        catalog,
        &EngineParams::default(),
        CandidateEvidence {
            post,
            feed_offset: FeedOffset::new(1),
            view_probability: ViewProbability::new(1.0).unwrap(),
            present: vec![ByteRange::new(0, 16)],
            stored_total: Some(16),
            continuation_source: None,
            independent_object_sources: Default::default(),
            recently_evicted: Vec::new(),
            in_flight: Vec::new(),
            origins: Vec::new(),
        },
    )
    .unwrap()
    .startup
    .unwrap()
}
async fn snapshot(binding: RepresentationBinding, name: &str) -> StoredMediaSnapshot {
    let store = PartialRangeStore::with_capacity(
        temp_directory(name),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    let post = binding.post().as_str().to_owned();
    store.bind_representation(binding).await.unwrap();
    store.set_total_len(&post, 16).await.unwrap();
    store.write_range(&post, 0, &[7; 16]).await.unwrap();
    store.media_snapshot(&post).await.unwrap()
}

fn meta(name: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://media.example/{name}.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
