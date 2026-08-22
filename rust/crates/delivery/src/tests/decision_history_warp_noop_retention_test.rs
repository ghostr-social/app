use super::work;
use crate::delivery_events::{command_channel, WarpDecisionPublication};
use ghostr_engine::adaptive::DecisionOutcome;

#[test]
fn terminal_warp_noops_remain_bounded_without_returning_tokens() {
    let work = work();
    let mut warp = work.warp.clone().expect("authoritative WARP decision");
    warp.selected = None;
    warp.evaluation = None;
    warp.search = Default::default();
    warp.admissible_action_ids.clear();
    warp.pruned_action_ids = warp
        .generated
        .actions
        .iter()
        .map(|action| action.node.id)
        .collect();
    let (handle, commands) = command_channel();
    for _ in 0..65 {
        let token = commands.publish_warp_decision(WarpDecisionPublication {
            snapshot: work.snapshot.as_ref().expect("planning snapshot"),
            decision: &warp,
            legacy_prices: work.shadow_prices,
            models: &work.decision_models,
        });
        assert!(token.is_none());
    }

    let history = handle.decision_history();
    assert_eq!(history.records.len(), 64);
    assert!(history.records.iter().all(|record| {
        record.schema_version == 3
            && record.eventual_outcome
                == DecisionOutcome::Succeeded {
                    bytes: 0,
                    elapsed_ms: 0,
                }
    }));
}
