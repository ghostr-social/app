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
