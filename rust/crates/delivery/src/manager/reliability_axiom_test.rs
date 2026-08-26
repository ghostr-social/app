use super::*;

pub(crate) async fn load_field_reliability(path: &Path) -> FieldReliabilityModel {
    match tokio::fs::read_to_string(path).await {
        Ok(json) => FieldReliabilityModel::from_json(&json).unwrap_or_default(),
        Err(_) => FieldReliabilityModel::default(),
    }
}

pub(crate) async fn save_field_reliability(
    path: &Path,
    model: &FieldReliabilityModel,
) -> io::Result<()> {
    save_json(path, model.to_json()).await
}
