use super::request_context;
use ghostr_engine::catalog::{Catalog, HttpObservation, LearnedFacts};
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::origin_model::{MediaClass, RequestMethod};
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const SOURCE: &str = "https://media.example/video.mp4";
const OBSERVED_AT_MS: u64 = 1_000;
const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

#[test]
fn request_context_uses_fused_range_evidence_at_plan_time() {
    let post = PostId::new("post");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), metadata(None));
    let identity = binding.transfer(SOURCE).expect("source identity");
    let response = HttpObservation::new(
        LearnedFacts {
            content_length: Some(2_000_000),
            accept_ranges: Some(false),
            host: None,
        },
        None,
        OBSERVED_AT_MS,
        EvidenceValidator::strong_etag("\"generation-1\""),
    );
    assert!(catalog.learn_response_observation_for(&identity, response));
    let entry = catalog.lookup(&post).expect("catalog entry");

    assert_eq!(
        request_context(entry, SOURCE, OBSERVED_AT_MS),
        (RequestMethod::FullGet, MediaClass::WholeObject, 2_000_000)
    );
    assert_eq!(
        request_context(entry, SOURCE, OBSERVED_AT_MS + DAY_MS),
        (
            RequestMethod::RangeGet,
            MediaClass::ProgressiveMp4,
            256 * 1024
        )
    );
}

#[test]
fn whole_context_uses_the_conservative_size_upper_bound() {
    let post = PostId::new("bounded");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(post.clone(), metadata(Some(20_000_000)));
    let identity = binding.transfer(SOURCE).expect("valid test fixture");
    let head = HttpObservation::new(
        LearnedFacts {
            content_length: Some(1_000_000),
            ..LearnedFacts::default()
        },
        None,
        OBSERVED_AT_MS,
        None,
    );
    assert!(catalog.learn_head_observation_for(&identity, head));
    let response = HttpObservation::new(
        LearnedFacts {
            accept_ranges: Some(false),
            ..LearnedFacts::default()
        },
        None,
        OBSERVED_AT_MS,
        None,
    );
    assert!(catalog.learn_action_response_observation_for(&identity, response));

    assert_eq!(
        request_context(
            catalog.lookup(&post).expect("valid test fixture"),
            SOURCE,
            OBSERVED_AT_MS
        ),
        (RequestMethod::FullGet, MediaClass::WholeObject, 20_000_000)
    );
}

fn metadata(size_bytes: Option<u64>) -> VideoMeta {
    VideoMeta {
        urls: vec![SOURCE.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes,
        duration_ms: Some(8_000),
    }
}
