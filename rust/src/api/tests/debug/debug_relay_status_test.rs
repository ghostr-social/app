use crate::api::debug::relay_status::snapshot;
use crate::discovery::cache::client_with_event_cache;

#[tokio::test]
async fn relay_snapshot_distinguishes_registered_and_missing_relays() {
    let client = client_with_event_cache();
    client
        .add_relay("wss://registered.example")
        .await
        .expect("register relay");
    let configured = vec![
        "wss://registered.example".to_owned(),
        "wss://missing.example".to_owned(),
    ];

    let relays = snapshot(&client, &configured).await;

    assert_eq!(relays[0].status, "initialized");
    assert_eq!(relays[1].status, "unavailable");
}
