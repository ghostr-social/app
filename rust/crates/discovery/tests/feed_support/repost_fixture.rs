use super::{signed_event, SignedEventFixture};
use ghostr_discovery::content::reposts::GENERIC_REPOST_KIND;
use nostr_sdk::{Event, JsonUtil, Keys, Kind};

/// A verified NIP-18 wrapper embedding the complete original event.
pub fn repost(keys: &Keys, original: &Event, created_at: u64) -> Event {
    let kind = if original.kind == Kind::TextNote {
        6
    } else {
        16
    };
    let mut tags = vec![
        vec![
            "e".to_owned(),
            original.id.to_hex(),
            "wss://relay.example".to_owned(),
        ],
        vec!["p".to_owned(), original.pubkey.to_hex()],
        vec!["k".to_owned(), original.kind.as_u16().to_string()],
    ];
    if let Some(identifier) = original.tags.identifier() {
        tags.push(vec![
            "a".to_owned(),
            format!(
                "{}:{}:{}",
                original.kind.as_u16(),
                original.pubkey.to_hex(),
                identifier
            ),
        ]);
    }
    signed_event(SignedEventFixture {
        keys,
        kind: Kind::Custom(kind),
        content: &original.as_json(),
        tags,
        created_at,
    })
}

/// A generic wrapper pinned to one exact addressable revision.
pub fn specific_repost(keys: &Keys, original: &Event, created_at: u64) -> Event {
    signed_event(SignedEventFixture {
        keys,
        kind: Kind::Custom(GENERIC_REPOST_KIND),
        content: &original.as_json(),
        tags: vec![
            vec!["e".to_owned(), original.id.to_hex()],
            vec!["p".to_owned(), original.pubkey.to_hex()],
            vec!["k".to_owned(), original.kind.as_u16().to_string()],
        ],
        created_at,
    })
}
