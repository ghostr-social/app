use ghostr_delivery::delivery_events::DecisionHistorySnapshot;
use ghostr_engine::adaptive::{DecisionOutcome, DecisionRecord, RecordedWarpCommand};

pub fn has_terminal_hedge(history: &DecisionHistorySnapshot, expected_bytes: u64) -> bool {
    let Some((hedge, primary)) = terminal_hedge(history, expected_bytes) else {
        return false;
    };
    hedge
        .chosen_action_id
        .is_some_and(|alternate| alternate != primary)
        && primary_loser(history, primary).is_some()
}

pub fn assert_exact_hedge(history: &DecisionHistorySnapshot, expected_bytes: u64) {
    let (hedge, primary) = terminal_hedge(history, expected_bytes).expect("terminal hedge record");
    let alternate = hedge.chosen_action_id.expect("bound alternate action");
    assert_ne!(alternate, primary, "hedge must bind its alternate action");
    primary_loser(history, primary).expect("cancelled primary loser");
}

fn primary_loser(history: &DecisionHistorySnapshot, primary: u64) -> Option<&DecisionRecord> {
    history.records.iter().find(|record| {
        record.chosen_action_id == Some(primary)
            && matches!(record.eventual_outcome, DecisionOutcome::Cancelled { .. })
    })
}

fn terminal_hedge(
    history: &DecisionHistorySnapshot,
    expected_bytes: u64,
) -> Option<(&DecisionRecord, u64)> {
    history.records.iter().find_map(|record| {
        let RecordedWarpCommand::Hedge {
            primary_action_id, ..
        } = selected_command(record)?
        else {
            return None;
        };
        matches!(
            record.eventual_outcome,
            DecisionOutcome::Succeeded { bytes, .. } if bytes == expected_bytes
        )
        .then_some((record, *primary_action_id))
    })
}

fn selected_command(record: &DecisionRecord) -> Option<&RecordedWarpCommand> {
    record
        .warp_decision
        .as_ref()?
        .selected
        .as_ref()
        .map(|selected| &selected.command)
}
