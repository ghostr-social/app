//! Protocol-level syntax checks for relay hints carried by repost tags.

use url::Url;

pub(super) fn valid_relay_hint(raw: &str) -> bool {
    Url::parse(raw).is_ok_and(|url| matches!(url.scheme(), "ws" | "wss") && url.host().is_some())
}
