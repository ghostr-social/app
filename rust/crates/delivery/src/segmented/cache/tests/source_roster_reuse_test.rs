use super::super::{SegmentedCache, SegmentedPhase};
use crate::segmented::prepare::PreparedObject;
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

#[test]
fn ready_bootstrap_survives_roster_changes_while_its_root_remains() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    let primary = "https://primary.example/index.m3u8";
    let backup = "https://backup.example/index.m3u8";
    cache.replace_focus(1, vec![(post.clone(), roots(primary, backup))]);
    store_ready(&cache, &post, 1, object(backup));

    cache.replace_focus(
        2,
        vec![(
            post.clone(),
            vec![
                backup.to_owned(),
                primary.to_owned(),
                "https://third.example/index.m3u8".to_owned(),
            ],
        )],
    );

    assert_eq!(cache.snapshot("post").phase, SegmentedPhase::Ready);
    assert!(cache.object(backup).is_some());
}

#[test]
fn ready_bootstrap_is_found_through_a_new_fragment_alias() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    let old = "https://origin.example/index.m3u8#old";
    let new = "https://origin.example/index.m3u8#new";
    cache.replace_focus(1, vec![(post.clone(), vec![old.to_owned()])]);
    store_ready(&cache, &post, 1, object(old));

    cache.replace_focus(2, vec![(post, vec![new.to_owned()])]);

    assert_eq!(cache.snapshot("post").phase, SegmentedPhase::Ready);
    assert!(cache.object(new).is_some());
}

pub(super) fn roots(primary: &str, backup: &str) -> Vec<String> {
    vec![primary.to_owned(), backup.to_owned()]
}

pub(super) fn object(root: &str) -> PreparedObject {
    PreparedObject {
        request_url: root.to_owned(),
        final_url: Url::parse(root).expect("valid test fixture"),
        body: Arc::from(b"#EXTM3U\n".as_slice()),
        content_type: Some("application/vnd.apple.mpegurl".to_owned()),
        cache: Default::default(),
    }
}

pub(super) fn store_ready(
    cache: &SegmentedCache,
    post: &PostId,
    generation: u64,
    object: PreparedObject,
) {
    assert!(cache.mark_stage_preparing(post, generation, 500, object.body.len() as u64));
    assert!(cache.store_stage_object(post, generation, object).is_some());
    assert!(cache.mark_stage_ready(post, generation));
}
