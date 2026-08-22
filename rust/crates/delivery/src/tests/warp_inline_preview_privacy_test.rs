use super::warp_inline_preview_plan_test::{plan, BLURHASH};
use ghostr_engine::adaptive::{
    DecisionPrivacy, DecisionRecord, ShadowPrices, WarpDecisionRecordInput,
};
use ghostr_engine::PreviewDescriptor;

#[test]
fn inline_blurhash_never_enters_the_decision_or_search_schema() {
    let work = plan(PreviewDescriptor::inline_blurhash(BLURHASH));
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 1,
        snapshot: work.snapshot.as_ref().unwrap(),
        decision: work.warp.as_ref().unwrap(),
        legacy_shadow_prices: ShadowPrices::new(0, 0, 0, 0),
        models: &work.decision_models,
        privacy: &DecisionPrivacy::from_key([7; 32]),
    });

    let json = serde_json::to_string(&record).unwrap();
    assert!(!json.contains(BLURHASH));
    assert!(!json.to_ascii_lowercase().contains("blurhash"));
}
