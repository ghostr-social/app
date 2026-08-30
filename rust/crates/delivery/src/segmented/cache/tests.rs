use super::{SegmentedCache, SegmentedPhase};
use crate::segmented::prepare::PreparedObject;
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

mod alias_replacement_test;
mod focus_reclamation_test;
mod generation_test;
mod http_freshness_test;
mod prepared_authority_aba_test;
mod prepared_authority_fixture;
mod prepared_authority_publication_test;
mod prepared_authority_retirement_test;
mod protected_shared_reclaim_test;
mod source_roster_owner_removal_test;
mod source_roster_reuse_test;
mod stage_admission_capacity_test;
mod stage_admission_insufficient_reclaim_test;
mod stage_admission_stale_reclaim_test;
mod stage_assembly_cancellation_test;
mod stage_capacity_fixture;
mod stage_lease_fixture;
mod stage_lease_focus_removal_test;
mod stage_lease_index_test;
mod stage_lease_invalidation_test;
mod stage_lease_stale_attempt_test;
mod stage_prehash_cancellation_test;
mod staged_assembly_capacity_test;
mod staged_block_storage_test;
mod staged_retry_test;
mod staged_same_source_retry_test;

#[test]
fn protected_bootstraps_are_never_silently_evicted_at_publication() {
    let cache = SegmentedCache::new();
    cache.replace_focus(
        1,
        ["first", "second", "third"]
            .into_iter()
            .map(|id| {
                (
                    PostId::new(id),
                    vec![format!("https://{id}.example/index.m3u8")],
                )
            })
            .collect(),
    );

    for post in ["first", "second"] {
        store_ready(&cache, &PostId::new(post), 1, prepared(post));
    }

    assert!(!cache.mark_stage_preparing(&PostId::new("third"), 1, 500, 8 * 1024 * 1024,));
    assert_eq!(cache.snapshot("first").phase, SegmentedPhase::Ready);
    assert_eq!(cache.snapshot("second").phase, SegmentedPhase::Ready);
    assert_eq!(cache.snapshot("third").phase, SegmentedPhase::Queued);
}

fn prepared(post: &str) -> Vec<PreparedObject> {
    let body: Arc<[u8]> = Arc::from(vec![0; 8 * 1024 * 1024]);
    ["index.m3u8", "segment.m4s"]
        .into_iter()
        .map(|name| PreparedObject {
            request_url: format!("https://{post}.example/{name}"),
            final_url: Url::parse(&format!("https://{post}.example/{name}"))
                .expect("valid test fixture"),
            body: std::sync::Arc::clone(&body),
            content_type: None,
            cache: Default::default(),
        })
        .collect()
}

fn store_ready(
    cache: &SegmentedCache,
    post: &PostId,
    generation: u64,
    objects: Vec<PreparedObject>,
) {
    for object in objects {
        assert!(cache.mark_stage_preparing(post, generation, 500, object.body.len() as u64));
        assert!(cache.store_stage_object(post, generation, object).is_some());
    }
    assert!(cache.mark_stage_ready(post, generation));
}
