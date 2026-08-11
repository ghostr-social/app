use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;

#[tokio::test]
async fn malformed_capabilities_fail_closed_for_use_and_release() {
    let capabilities = ProgressiveCapabilities::production();

    assert!(!capabilities.authorizes("not-a-capability", "clip").await);
    assert!(!capabilities.release("not-a-capability").await);
}
