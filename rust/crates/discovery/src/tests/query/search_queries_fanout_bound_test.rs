use crate::query::search::{resolve_relays, RelayTarget};
use crate::relay::roles::MAX_RELAY_READ_FANOUT;

#[test]
fn ordinary_relay_resolution_keeps_priority_order_inside_the_fanout_bound() {
    let hints = relays("hint", 8);
    let mut search = hints[..2].to_vec();
    search.extend(relays("search", 6));
    let mut outbox = search[2..4].to_vec();
    outbox.extend(relays("outbox", 22));
    let target = RelayTarget::HintedRelays(hints.clone());

    let resolved = resolve_relays(&target, &search, Some(&outbox)).expect("valid test fixture");

    assert_eq!(resolved.len(), MAX_RELAY_READ_FANOUT);
    assert_eq!(&resolved[..8], hints);
    assert_eq!(&resolved[8..14], &search[2..]);
    assert_eq!(&resolved[14..], &outbox[2..20]);
    assert!(!resolved.contains(&outbox[20]));
}

fn relays(prefix: &str, count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("wss://{prefix}-{index}.example"))
        .collect()
}
