use crate::api::engine_control::validated_relay_urls;

#[test]
fn configured_relay_urls_are_normalized_and_deduplicated_in_order() {
    let urls = validated_relay_urls(vec![
        " WSS://Relay.Example:443/ ".to_owned(),
        "wss://relay.example".to_owned(),
        "wss://other.example/path/".to_owned(),
    ])
    .expect("valid relay configuration");

    assert_eq!(
        urls,
        vec!["wss://relay.example", "wss://other.example/path"]
    );
}
