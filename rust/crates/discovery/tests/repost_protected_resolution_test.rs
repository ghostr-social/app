use crate::content::candidates::CandidateRegistry;
use crate::tests::feed_support::{signed_event, SignedEventFixture};
use nostr_sdk::{Keys, Kind};

#[test]
fn empty_repost_resolves_a_verified_protected_original() {
    let creator = Keys::generate();
    let original = signed_event(SignedEventFixture {
        keys: &creator,
        kind: Kind::Custom(21),
        content: "https://cdn.example/protected.mp4",
        tags: vec![vec!["-".to_owned()]],
        created_at: 10,
    });
    let wrapper = signed_event(SignedEventFixture {
        keys: &Keys::generate(),
        kind: Kind::Custom(16),
        content: "",
        tags: vec![
            vec![
                "e".to_owned(),
                original.id.to_hex(),
                "wss://relay.example".to_owned(),
            ],
            vec!["p".to_owned(), original.pubkey.to_hex()],
            vec!["k".to_owned(), "21".to_owned()],
        ],
        created_at: 20,
    });

    let batch = CandidateRegistry::new().inspect_all(&[wrapper.clone(), original]);
    let repost = batch
        .posts
        .iter()
        .find(|post| post.repost.is_some())
        .expect("resolved repost");

    assert!(repost.is_protected);
    assert!(repost.signed_event_json.is_none());
    assert_eq!(repost.feed_sort_at, 20);
    assert_eq!(
        repost.repost.as_ref().expect("valid test fixture").event_id,
        wrapper.id.to_hex()
    );
}
