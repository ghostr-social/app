use crate::relay::url::normalize_untrusted_relay_url;

#[test]
fn untrusted_relay_hints_reject_local_and_ip_network_targets() {
    for raw in [
        "ws://localhost:7000",
        "wss://localhost/socket",
        "wss://127.0.0.1/socket",
        "wss://relay.local/socket",
        "wss://localhost./socket",
        "wss://relay.internal./socket",
    ] {
        assert_eq!(normalize_untrusted_relay_url(raw), None, "{raw}");
    }
    assert_eq!(
        normalize_untrusted_relay_url("WSS://Relay.Example:443/socket/"),
        Some("wss://relay.example/socket".to_owned())
    );
}
