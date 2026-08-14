use crate::content::repost_reference::reference_for_repost;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag};
use nostr_sdk::Keys;

#[test]
fn coordinate_metadata_must_match_the_address() {
    let creator = Keys::generate();
    let other = Keys::generate();
    let reposter = Keys::generate();
    let coordinate = format!("34235:{}:clip", creator.public_key());
    let wrong_author = signed_wrapper(
        &reposter,
        16,
        "",
        vec![
            tag(&["a", &coordinate]),
            tag(&["p", &other.public_key().to_hex()]),
        ],
    );
    let wrong_kind = signed_wrapper(
        &reposter,
        16,
        "",
        vec![tag(&["a", &coordinate]), tag(&["k", "30023"])],
    );

    assert!(reference_for_repost(&wrong_author).is_none());
    assert!(reference_for_repost(&wrong_kind).is_none());
}
