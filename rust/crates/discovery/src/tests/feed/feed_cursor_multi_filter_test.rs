use crate::feed::cursor::retrieval_cursor;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};

#[test]
fn cursor_never_skips_the_shallowest_search_filter() {
    let events = vec![
        event(Kind::Custom(21), 100),
        event(Kind::Custom(21), 90),
        event(Kind::TextNote, 100),
        event(Kind::TextNote, 50),
        event(Kind::Custom(1063), 100),
        event(Kind::Custom(1063), 20),
        event(Kind::Metadata, 1),
    ];

    assert_eq!(retrieval_cursor(&events), Some(Timestamp::from(89)));
}

fn event(kind: Kind, created_at: u64) -> Event {
    EventBuilder::new(kind, "event")
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("signed")
}
