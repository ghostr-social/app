use crate::content::reposts::{resolved_repost, RepostTarget};
use crate::tests::repost_reference_fixture::{signed_wrapper, video};
use nostr_sdk::{Keys, Kind};

#[test]
fn resolved_repost_rejects_unrepresentable_kind_and_coordinate_states() {
    let creator = Keys::generate();
    let reposter = Keys::generate();
    let wrapper = signed_wrapper(&reposter, 16, "", vec![]);
    let generic = video(&creator, Kind::Custom(21), vec![]);
    let addressable_without_identifier = video(&creator, Kind::Custom(34235), vec![]);

    assert!(resolved_repost(&wrapper, &generic, RepostTarget::SpecificEvent, 99).is_none());
    assert!(resolved_repost(&wrapper, &generic, RepostTarget::Coordinate, 16).is_none());
    assert!(resolved_repost(
        &wrapper,
        &addressable_without_identifier,
        RepostTarget::Coordinate,
        16,
    )
    .is_none());
}
