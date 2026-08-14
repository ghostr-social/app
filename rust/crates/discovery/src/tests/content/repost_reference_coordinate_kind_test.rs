use crate::content::repost_reference::reference_for_repost;
use crate::tests::repost_reference_fixture::{signed_wrapper, tag};
use nostr_sdk::Keys;

#[test]
fn coordinate_references_require_generic_and_addressable_kinds() {
    let author = Keys::generate();
    let coordinate = format!("34235:{}:clip", author.public_key());
    let specific = signed_wrapper(&author, 6, "", vec![tag(&["a", &coordinate])]);
    let non_addressable = signed_wrapper(
        &author,
        16,
        "",
        vec![tag(&["a", &format!("1:{}:clip", author.public_key())])],
    );

    assert!(reference_for_repost(&specific).is_none());
    assert!(reference_for_repost(&non_addressable).is_none());
}
