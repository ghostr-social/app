use super::{SegmentedCache, SegmentedPhase};
use crate::segmented::prepare::PreparedObject;
use crate::segmented::PreparedHls;
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

mod alias_replacement_test;
mod generation_test;

#[test]
fn evicting_bootstrap_bytes_revokes_stale_readiness() {
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

    for post in ["first", "second", "third"] {
        cache.complete(&PostId::new(post), 1, Ok(prepared(post)));
    }

    assert_eq!(cache.snapshot("first").phase, SegmentedPhase::Queued);
    assert_eq!(cache.snapshot("third").phase, SegmentedPhase::Ready);
}

fn prepared(post: &str) -> PreparedHls {
    let body: Arc<[u8]> = Arc::from(vec![0; 8 * 1024 * 1024]);
    PreparedHls {
        objects: ["index.m3u8", "segment.m4s"]
            .into_iter()
            .map(|name| PreparedObject {
                request_url: format!("https://{post}.example/{name}"),
                final_url: Url::parse(&format!("https://{post}.example/{name}")).unwrap(),
                body: body.clone(),
                content_type: None,
            })
            .collect(),
    }
}
