use crate::execution::collector::collect_page;
use crate::execution::fetch::FetchedEvents;
use crate::query::search::QueryRole;
use nostr_sdk::{EventBuilder, Keys, Kind, Timestamp};

#[tokio::test]
async fn epoch_wire_page_exhausts_instead_of_repeating() {
    let event = EventBuilder::new(Kind::Custom(21), "https://cdn.example/v.mp4")
        .custom_created_at(Timestamp::from(0))
        .sign_with_keys(&Keys::generate())
        .expect("signed event");
    let fetch = tokio::spawn(async move { Ok(FetchedEvents::fresh(vec![event])) });

    let page = collect_page(vec![(QueryRole::Primary, fetch)])
        .await
        .expect("page");

    assert_eq!(page.events.len(), 1);
    assert_eq!(page.cursor, None);
}

#[tokio::test]
async fn incomplete_filter_prevents_a_sibling_cursor_commit() {
    let complete = event_at(200);
    let incomplete = event_at(100);
    let complete = tokio::spawn(async move { Ok(FetchedEvents::fresh(vec![complete])) });
    let incomplete = tokio::spawn(async move {
        Ok(FetchedEvents {
            events: vec![incomplete],
            fresh_boundary: None,
            wire_complete: false,
        })
    });

    let page = collect_page(vec![
        (QueryRole::Primary, complete),
        (QueryRole::Additive, incomplete),
    ])
    .await
    .expect("partial union remains usable");

    assert_eq!(page.events.len(), 2);
    assert_eq!(page.cursor, None);
}

fn event_at(created_at: u64) -> nostr_sdk::Event {
    EventBuilder::new(Kind::Custom(21), "https://cdn.example/v.mp4")
        .custom_created_at(Timestamp::from(created_at))
        .sign_with_keys(&Keys::generate())
        .expect("signed event")
}
