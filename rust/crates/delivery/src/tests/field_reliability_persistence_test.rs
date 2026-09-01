use crate::manager::reliability::axiom_test_support::load_field_reliability;
use crate::manager::reliability::axiom_test_support::save_field_reliability;

use ghostr_engine::evidence::{
    CalibrationContext, CalibrationDimensions, CalibrationLabel, EvidenceField,
    FieldReliabilityModel,
};

#[tokio::test]
async fn field_correctness_learning_survives_restart_without_raw_event_data() {
    let path = std::env::temp_dir().join(format!("field-reliability-{}.json", std::process::id()));
    let dimensions = CalibrationDimensions::new(
        Some("issuer".into()),
        Some("cdn.example".into()),
        Some("https://cdn.example/video.mp4".into()),
    );
    let context = CalibrationContext::new(dimensions, EvidenceField::Size, "nostr");
    let mut expected = FieldReliabilityModel::default();
    expected.observe(CalibrationLabel::new(context.clone(), false, 10));

    save_field_reliability(&path, &expected)
        .await
        .expect("valid test fixture");
    let restored = load_field_reliability(&path).await;

    assert_eq!(
        restored.estimate(&context, 10),
        expected.estimate(&context, 10)
    );
    let json = tokio::fs::read_to_string(&path)
        .await
        .expect("valid test fixture");
    assert!(!json.contains("raw_event"));
    let _ = tokio::fs::remove_file(path).await;
}
