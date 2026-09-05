use sha2::{Digest as _, Sha256};
pub(super) async fn replace_generation_fingerprint(root: &std::path::Path) {
    let path = root.join("clip.generation.json");
    let mut value: serde_json::Value =
        serde_json::from_slice(&tokio::fs::read(&path).await.expect("fixture")).expect("fixture");
    value["representation"] = "foreign-representation".into();
    tokio::fs::write(path, serde_json::to_vec(&value).expect("fixture"))
        .await
        .expect("fixture");
}

pub(super) async fn install_blocked_policy_cleanup(root: &std::path::Path) {
    let manifest = tokio::fs::read(root.join("clip.ranges.json"))
        .await
        .expect("fixture");
    let old_hash = format!("{:x}", Sha256::digest(&manifest));
    let intent = format!(
        r#"{{"version":2,"old_accounted":8,"new_accounted":4,"old_manifest_sha256":"{old_hash}"}}"#
    );
    tokio::fs::write(root.join("clip.evict.intent"), intent)
        .await
        .expect("fixture");
    tokio::fs::create_dir(root.join("clip.part.evict"))
        .await
        .expect("fixture");
}
