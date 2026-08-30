use crate::api::delivery::snapshots::{event_for, hls_snapshot};
use crate::api::tests::hls_runtime_support;

#[tokio::test]
async fn structural_hls_readiness_carries_only_its_exact_hls_authority() {
    let (runtime, native, root) = hls_runtime_support::prepared().await;
    let authority = native.authority.clone().expect("cache authority");
    let representation = authority.representation_id().fingerprint().to_owned();
    let revision = authority.asset_revision().value();

    let snapshot = hls_snapshot(native);
    assert!(snapshot.startable, "bootstrap is structurally startable");
    let event = event_for("stream", None, snapshot).expect("readiness event");

    assert!(event.startable);
    assert_eq!(event.representation_id, None, "not progressive authority");
    assert_eq!(event.asset_id, None, "not a progressive capability");
    assert_eq!(event.hls_delivery_id.as_deref(), Some("stream"));
    assert_eq!(event.hls_representation_id, Some(representation));
    assert_eq!(event.hls_asset_revision, Some(revision));

    drop(runtime);
    std::fs::remove_dir_all(root).ok();
}
