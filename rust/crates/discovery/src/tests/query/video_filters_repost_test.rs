use serde_json::json;

use crate::query::video_filters::{discovery_filters, DiscoveryRequest, RepostAdmission};
use crate::tests::support::{author, filter_json, AUTHOR_A};

#[test]
fn repost_admission_adds_scoped_wrapper_and_deletion_filters() {
    let request = DiscoveryRequest {
        authors: vec![author(AUTHOR_A)],
        reposts: RepostAdmission::Included,
        ..DiscoveryRequest::default()
    };

    let filters = discovery_filters(&request);

    assert_eq!(filters.len(), 7);
    assert_eq!(filter_json(&filters[4])["kinds"], json!([6]));
    assert_eq!(filter_json(&filters[5])["kinds"], json!([16]));
    assert_eq!(filter_json(&filters[6])["kinds"], json!([5]));
    for filter in &filters[4..] {
        assert_eq!(filter_json(filter)["authors"], json!([AUTHOR_A]));
    }
}
