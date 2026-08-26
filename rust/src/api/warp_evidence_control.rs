use crate::api::runtime::registry;
use flutter_rust_bridge::frb;

const MAX_PLAN_PAGE: u32 = 64;

#[frb]
pub fn ffi_warp_evidence_page_json(after_plan_revision: u64, limit: u32) -> anyhow::Result<String> {
    anyhow::ensure!(
        (1..=MAX_PLAN_PAGE).contains(&limit),
        "invalid plan page limit"
    );
    let delivery = registry::engine()?.gateway.delivery();
    Ok(delivery.evidence_page_json(after_plan_revision, limit as usize)?)
}

#[frb]
pub fn ffi_warp_decision_history_json() -> anyhow::Result<String> {
    let delivery = registry::engine()?.gateway.delivery();
    Ok(delivery.decision_history_json()?)
}
