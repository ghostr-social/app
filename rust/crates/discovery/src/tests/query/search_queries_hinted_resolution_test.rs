use crate::query::search::{resolve_relays, RelayTarget};

#[test]
fn hinted_target_merges_hint_search_and_outbox_without_duplicates() {
    let target =
        RelayTarget::HintedRelays(vec!["wss://hint".to_owned(), "wss://shared".to_owned()]);
    let search = vec!["wss://shared".to_owned(), "wss://search".to_owned()];
    let outbox = vec!["wss://search".to_owned(), "wss://write".to_owned()];

    let resolved = resolve_relays(&target, &search, Some(&outbox));

    assert_eq!(
        resolved,
        Some(vec![
            "wss://hint".to_owned(),
            "wss://shared".to_owned(),
            "wss://search".to_owned(),
            "wss://write".to_owned(),
        ])
    );
}
