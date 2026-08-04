//! Shared builders for discovery tests.

use nostr_sdk::{Filter, PublicKey};
use serde_json::Value;

/// Known-valid x-only public keys (secp256k1 G and 2G x-coordinates).
pub const AUTHOR_A: &str =
    "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
pub const AUTHOR_B: &str =
    "c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5";

pub fn author(hex: &str) -> PublicKey {
    PublicKey::from_hex(hex).expect("valid public key")
}

/// Relay-visible JSON form of a filter (what actually goes on the wire).
pub fn filter_json(filter: &Filter) -> Value {
    serde_json::to_value(filter).expect("filter serializes")
}
