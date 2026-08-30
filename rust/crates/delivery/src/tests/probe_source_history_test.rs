use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::probe::pool::{MetadataProbePool, ProbeClaimQuery};
use ghostr_engine::adaptive::ProbeClaimRefusal;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use std::collections::HashSet;

const FIRST: &str = "https://first.example/video.mp4";
const SECOND: &str = "https://second.example/video.mp4";

#[test]
fn completed_head_history_is_scoped_to_the_probed_source() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    catalog.upsert(post.clone(), metadata());
    let retry = RetryBook::new(RetryPolicy::default());
    let mut probes = MetadataProbePool::new(1);

    let first = probes
        .claim_selected(query(&catalog, &retry, &post, FIRST))
        .expect("first source probe");
    probes.learned(&first, None);

    assert_eq!(
        probes.claim_selected(query(&catalog, &retry, &post, FIRST)),
        Err(ProbeClaimRefusal::AlreadyProbed)
    );
    assert!(probes
        .claim_selected(query(&catalog, &retry, &post, SECOND))
        .is_ok());
}

#[test]
fn another_sources_completed_head_does_not_suppress_planner_discovery() {
    use crate::tests::warp_head_probe_context_fixture::{
        ahead_state_with_sources, generates_head_for, plan_at,
    };

    let post = PostId::new("post");
    let state = ahead_state_with_sources(post.clone(), vec![SECOND.into(), FIRST.into()]);
    let first = state
        .catalog()
        .transfer_identity(&post, FIRST)
        .expect("valid test fixture");
    let completed = HashSet::from([first]);

    assert!(generates_head_for(
        plan_at(&state, &[], &completed, 1, 2),
        &post
    ));
}

fn query<'a>(
    catalog: &'a Catalog,
    retry: &'a RetryBook,
    post: &'a PostId,
    source: &'a str,
) -> ProbeClaimQuery<'a> {
    ProbeClaimQuery {
        catalog,
        retry,
        post,
        source,
        observed_at_ms: 0,
    }
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![FIRST.into(), SECOND.into()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: None,
        duration_ms: None,
    }
}
