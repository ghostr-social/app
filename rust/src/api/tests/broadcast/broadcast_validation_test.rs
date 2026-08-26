//! `verified_event`: only pre-signed events whose id and signature
//! verify may cross the broadcast boundary — keys never do.

use crate::api::broadcast_control::verified_event;
use crate::api::tests::feed_fixtures::video_note;
use nostr_sdk::{JsonUtil as _, Keys};

#[test]
fn a_correctly_signed_event_passes() {
    let keys = Keys::generate();
    let event = video_note(&keys, "clip", 40);
    let verified = verified_event(&event.as_json()).expect("valid event verifies");
    assert_eq!(verified.id, event.id);
}

#[test]
fn a_tampered_event_is_rejected() {
    let keys = Keys::generate();
    let event = video_note(&keys, "clip", 40);
    let tampered = event.as_json().replace("clip", "evil");
    assert!(verified_event(&tampered).is_err());
}

#[test]
fn malformed_json_is_rejected() {
    assert!(verified_event("not an event").is_err());
    assert!(verified_event("{}").is_err());
}
