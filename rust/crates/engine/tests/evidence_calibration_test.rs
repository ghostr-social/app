use ghostr_engine::evidence::{
    CalibrationContext, CalibrationDimensions, CalibrationLabel, EvidenceField,
    FieldReliabilityModel,
};

#[test]
fn recent_exact_context_labels_outweigh_old_global_metadata_agreement() {
    let mut model = FieldReliabilityModel::default();
    let exact = context(Some("issuer-a"), Some("cdn.example"), Some(URL));
    let other = context(Some("issuer-b"), Some("other.example"), None);
    for at in 0..8 {
        model.observe(CalibrationLabel::new(other.clone(), true, at));
    }
    model.observe(CalibrationLabel::new(exact.clone(), true, 1));
    model.observe(CalibrationLabel::new(exact.clone(), false, 100_000));

    let estimate = model.estimate(&exact, 100_000);
    assert!(
        estimate.mean_bps < 5_000,
        "recent contradiction must dominate"
    );
    assert!(estimate.lower_bound_bps <= estimate.mean_bps);
    assert!(estimate.effective_samples_bps > 0);
}

#[test]
fn unseen_url_inherits_discounted_issuer_reliability() {
    let mut model = FieldReliabilityModel::default();
    let known = context(Some("issuer-a"), Some("known.example"), Some(URL));
    for at in 1..=4 {
        model.observe(CalibrationLabel::new(known.clone(), true, at));
    }
    let unseen = context(
        Some("issuer-a"),
        Some("new.example"),
        Some("https://new.example/video.mp4"),
    );

    let estimate = model.estimate(&unseen, 4);

    assert!(estimate.mean_bps > 5_000);
    assert!(estimate.effective_samples_bps > 0);
}

const URL: &str = "https://cdn.example/video.mp4";

fn context(issuer: Option<&str>, origin: Option<&str>, url: Option<&str>) -> CalibrationContext {
    let dimensions = CalibrationDimensions::new(
        issuer.map(str::to_owned),
        origin.map(str::to_owned),
        url.map(str::to_owned),
    );
    CalibrationContext::new(dimensions, EvidenceField::Size, "progressive")
}
