use crate::query::video_filters::{discovery_filters, DiscoveryRequest, RepostAdmission};
use crate::tests::support::filter_json;

#[test]
fn generic_reposts_allow_wrappers_without_the_optional_kind_tag() {
    let request = DiscoveryRequest {
        reposts: RepostAdmission::Included,
        ..DiscoveryRequest::default()
    };

    let generic = filter_json(&discovery_filters(&request)[5]);

    assert!(generic.get("#k").is_none());
}
