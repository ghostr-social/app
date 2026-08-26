use crate::segmented::cache::axiom_test_support::{StageBlock, StoredStage};
use crate::segmented::cache::StageReservation;
use crate::segmented::prepare::PreparedObject;
use crate::segmented::SegmentedCache;
use ghostr_engine::PostId;
use std::sync::Arc;
use url::Url;

#[test]
fn partial_blocks_remain_shared_until_one_final_assembly() {
    let cache = SegmentedCache::new();
    let post = PostId::new("post");
    cache.replace_focus(1, vec![(post.clone(), vec!["source".to_owned()])]);
    let bodies: Vec<Arc<[u8]>> = [b"aaaa", b"bbbb", b"cccc", b"dddd"]
        .map(|bytes| Arc::from(bytes.as_slice()))
        .to_vec();

    for (index, body) in bodies.iter().take(3).enumerate() {
        assert!(cache.mark_stage_preparing(&post, 1, 500, body.len() as u64));
        assert!(matches!(
            cache.store_stage_block(
                &post,
                1,
                StageBlock::partial((index * 4) as u64, object(std::sync::Arc::clone(body)))
            ),
            Some(StoredStage::Partial)
        ));
    }
    assert!(bodies
        .iter()
        .take(3)
        .all(|body| Arc::strong_count(body) == 2));
    let reservation = StageReservation::final_block(4, 16).expect("valid test fixture");
    assert!(cache.mark_stage_preparing(&post, 1, 500, reservation));
    let Some(StoredStage::Complete(completed)) = cache.store_stage_block(
        &post,
        1,
        StageBlock::complete(12, object(std::sync::Arc::clone(&bodies[3]))),
    ) else {
        panic!("final block assembles the staged object");
    };

    assert_eq!(completed.object.body.as_ref(), b"aaaabbbbccccdddd");
    assert!(bodies
        .iter()
        .take(3)
        .all(|body| Arc::strong_count(body) == 2));
    assert!(cache.commit_stage_complete(&post, 1, *completed));
    assert!(bodies.iter().all(|body| Arc::strong_count(body) == 1));
}

fn object(body: Arc<[u8]>) -> PreparedObject {
    PreparedObject {
        request_url: "source".to_owned(),
        final_url: Url::parse("https://cdn.example/source").expect("valid test fixture"),
        body,
        content_type: None,
        cache: Default::default(),
    }
}
