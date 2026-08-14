use crate::api::feed::mapping::parse_feed_spec;
use crate::api::feed_types::{FfiFeedKind, FfiFeedSpec};
use crate::discovery::feed::spec::FeedSpec;
use nostr_sdk::Keys;

#[test]
fn following_is_distinct_from_a_profile_grid() {
    let followed = Keys::generate().public_key();
    let viewer = Keys::generate().public_key();
    let spec = FfiFeedSpec {
        kind: FfiFeedKind::Following,
        value: None,
        creators: vec![followed.to_hex()],
        viewer_pubkey: Some(viewer.to_hex()),
    };

    assert_eq!(
        parse_feed_spec(&spec).expect("following parses"),
        FeedSpec::Following {
            viewer: Some(viewer),
            follows: vec![followed],
        },
    );
}
