use super::types::ClientCapabilityState;
use super::ClientCapabilityModel;
use std::io;
use std::path::Path;

pub(crate) async fn load_client_capabilities(path: &Path) -> ClientCapabilityModel {
    let Ok(json) = tokio::fs::read_to_string(path).await else {
        return ClientCapabilityModel::default();
    };
    serde_json::from_str::<ClientCapabilityState>(&json)
        .map(ClientCapabilityModel::from_state)
        .unwrap_or_default()
}

pub(crate) async fn save_client_capabilities(
    path: &Path,
    model: &ClientCapabilityModel,
) -> io::Result<()> {
    let staging = path.with_extension("json.tmp");
    let body = serde_json::to_vec(&model.state()).expect("capability state always serializes");
    tokio::fs::write(&staging, body).await?;
    if let Err(error) = tokio::fs::rename(&staging, path).await {
        let _ = tokio::fs::remove_file(&staging).await;
        return Err(error);
    }
    Ok(())
}
