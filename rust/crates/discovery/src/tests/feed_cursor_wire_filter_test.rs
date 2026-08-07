//! Independent wire filters keep independent pagination boundaries.

use crate::relay_plan_collector::{collect_page, FetchHandle};
use crate::search_queries::QueryRole;
use nostr_sdk::{Event, EventBuilder, Keys, Kind, Timestamp};

#[tokio::test]
async fn shallow_general_note_bounds_a_deeper_mp4_hunt() {
    let page = collect_page(vec![
        fetch(QueryRole::Primary, event(Kind::Custom(21), "video", 80)),
        fetch(QueryRole::Additive, event(Kind::TextNote, "general", 100)),
        fetch(QueryRole::Additive, event(Kind::TextNote, "clip.mp4", 50)),
        fetch(QueryRole::Additive, event(Kind::Custom(1063), "file", 70)),
    ])
    .await
    .expect("wire pages");

    assert_eq!(page.cursor, Some(Timestamp::from(99)));
}

fn fetch(role: QueryRole, event: Event) -> (QueryRole, FetchHandle) {
    (role, tokio::spawn(async move { Ok(vec![event]) }))
}

fn event(kind: Kind, content: &str, created_at: u64) -> Event {
    EventBuilder::new(kind, content)
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("signed")
}
