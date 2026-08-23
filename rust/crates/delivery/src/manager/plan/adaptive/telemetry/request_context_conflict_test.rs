use super::request_context;
use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::origin_model::{MediaClass, RequestMethod};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";

#[test]
fn response_length_conflict_keeps_the_conservative_whole_request_size() {
    let post = PostId::new("conflicting-response-size");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), metadata());
    let identity = binding.transfer(SOURCE).expect("source identity");
    let response = HttpObservation::new(
        LearnedFacts {
            content_length: Some(1_000_000),
            accept_ranges: Some(false),
            host: None,
        },
        None,
        1_000,
        None,
    );
    assert!(catalog.learn_action_response_observation_for(&identity, response));

    assert_eq!(
        request_context(catalog.lookup(&post).unwrap(), SOURCE, 1_000),
        (RequestMethod::FullGet, MediaClass::WholeObject, 20_000_000)
    );
}

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(20_000_000),
        duration_ms: Some(8_000),
    }
}
