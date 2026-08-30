#[path = "partial_range_staged_stale_validator_revision_test/support/mod.rs"]
mod support;

#[tokio::test]
async fn stale_validator_cannot_preserve_revision_across_changed_sparse_bytes() {
    let fixture = support::seeded_fixture().await;
    let before = fixture.revision().await;

    fixture.publish_full_body(support::NEW_BODY).await;
    let after = fixture.revision().await;
    let current = fixture.current_bytes(support::NEW_BODY.len()).await;
    let stale_lease_rejected = fixture.stale_lease_rejected(support::NEW_BODY.len()).await;
    fixture.cleanup();

    assert_ne!(after, before);
    assert_eq!(current, support::NEW_BODY);
    assert!(stale_lease_rejected);
}

#[tokio::test]
async fn stale_validator_cannot_survive_a_changed_total_length() {
    let fixture = support::seeded_fixture().await;
    let before = fixture.revision().await;

    fixture.publish_full_body(support::LONG_BODY).await;
    let after = fixture.revision().await;
    let current = fixture.current_bytes(support::LONG_BODY.len()).await;
    let stale_lease_rejected = fixture.stale_lease_rejected(support::LONG_BODY.len()).await;
    fixture.cleanup();

    assert_ne!(after, before);
    assert_eq!(current, support::LONG_BODY);
    assert!(stale_lease_rejected);
}
