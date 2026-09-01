use crate::client_capability::{
    load_client_capabilities, save_client_capabilities, CapabilityAttempt, CapabilityEvent,
    CapabilityObservation, CapabilitySignal, ClientCapabilityModel, ClientCapabilityProfile,
    ClientCapabilityStatus,
};

#[tokio::test]
async fn capability_profile_survives_restart_and_corruption_fails_closed() {
    let root = std::env::temp_dir().join(format!("client-capability-{}", std::process::id()));
    let path = root.join("client_capability.json");
    tokio::fs::create_dir_all(&root)
        .await
        .expect("valid test fixture");
    let profile = ClientCapabilityProfile::try_new("fingerprint", None, None)
        .expect("valid test fixture")
        .with_persistent_identity(true);
    let mut model = ClientCapabilityModel::default();
    let attempt = CapabilityAttempt::new(1, 1);
    observe(
        &mut model,
        profile.clone(),
        attempt,
        CapabilitySignal::Initializing,
    );
    observe(
        &mut model,
        profile.clone(),
        attempt,
        CapabilitySignal::FirstFrameRendered,
    );

    save_client_capabilities(&path, &model)
        .await
        .expect("valid test fixture");
    let restored = load_client_capabilities(&path).await;
    assert!(matches!(
        restored.status(7, &profile),
        ClientCapabilityStatus::Supported { .. }
    ));
    tokio::fs::write(&path, "not json")
        .await
        .expect("valid test fixture");
    assert_eq!(
        load_client_capabilities(&path).await.status(7, &profile),
        ClientCapabilityStatus::Unknown,
    );
    std::fs::remove_dir_all(root).ok();
}

fn observe(
    model: &mut ClientCapabilityModel,
    profile: ClientCapabilityProfile,
    attempt: CapabilityAttempt,
    signal: CapabilitySignal,
) {
    model.observe(CapabilityObservation::new(
        7,
        attempt,
        profile,
        CapabilityEvent::new(10, signal),
    ));
}
