use crate::catalog::{Catalog, HttpObservation, LearnedFacts};
use crate::evidence::EvidenceValidator;
use crate::representation::RequestSelection;
use crate::tests::support::progressive_meta;
use crate::PostId;

#[test]
fn changing_request_selection_changes_http_authority_even_with_the_same_etag() {
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), progressive_meta(Some(8), None));
    let identity = binding
        .transfer("https://host.example/video.mp4")
        .expect("identity");
    assert!(catalog.learn_response_observation_for(&identity, observed(1)));
    let first = catalog
        .http_generation_for(&identity)
        .expect("first authority");
    assert_eq!(
        first.key().request_selection(),
        Some(RequestSelection::new([1; 32]))
    );
    assert!(catalog.learn_response_observation_for(&identity, observed(2)));
    let second = catalog
        .http_generation_for(&identity)
        .expect("second authority");
    assert_ne!(first, second);
    assert_eq!(
        second.key().request_selection(),
        Some(RequestSelection::new([2; 32]))
    );
}

fn observed(selection: u8) -> HttpObservation {
    HttpObservation::new(
        LearnedFacts {
            content_length: Some(8),
            accept_ranges: Some(true),
            host: None,
        },
        None,
        u64::from(selection),
        EvidenceValidator::strong_etag("\"v1\""),
    )
    .with_request_selection(Some(RequestSelection::new([selection; 32])))
}
