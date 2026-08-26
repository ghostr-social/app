use crate::delivery_events::command_channel;
use crate::tests::decision_log_fixture::{
    head_identity, head_work, selected, selected_head, selected_warp, work, wrong_post_identity,
    wrong_source_identity,
};
use ghostr_engine::adaptive::{DecisionPrivacy, DecisionRecord, WarpDecisionRecordInput};

#[test]
fn only_schema_two_head_decisions_can_be_claimed_as_probes() {
    let work = work();
    let identity = head_identity();
    let (legacy_handle, legacy_commands) = command_channel();
    let (_, legacy) = selected(&legacy_handle, &legacy_commands, &work);
    assert!(legacy_commands
        .claim_decision(legacy, &identity, 100)
        .is_err());

    let (warp_handle, warp_commands) = command_channel();
    let (_, transfer) = selected_warp(&warp_handle, &warp_commands, &work);
    assert!(warp_commands
        .claim_decision(transfer, &identity, 100)
        .is_err());

    let (head_handle, head_commands) = command_channel();
    let (_, head) = selected_head(&head_handle, &head_commands);
    let head = rejected(head, &wrong_post_identity(), &head_commands);
    let head = rejected(head, &wrong_source_identity(), &head_commands);
    let claim = head_commands
        .claim_decision(head, &identity, 100)
        .unwrap_or_else(|_| panic!("matching HEAD identity"));
    drop(claim);

    malformed_head_records_are_not_claimable(&identity);
}

fn malformed_head_records_are_not_claimable(
    identity: &ghostr_engine::representation::TransferIdentity,
) {
    let work = head_work();
    let privacy = DecisionPrivacy::from_key([7; 32]);
    let mut record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 1,
        snapshot: work.snapshot.as_ref().expect("valid test fixture"),
        decision: work.warp.as_ref().expect("valid test fixture"),
        legacy_shadow_prices: work.shadow_prices,
        models: &work.decision_models,
        privacy: &privacy,
    });
    assert!(record.authorizes_probe_claim(identity, &privacy));
    record.schema_version = 1;
    assert!(!record.authorizes_probe_claim(identity, &privacy));
    record.schema_version = 2;
    record.chosen_action = None;
    assert!(!record.authorizes_probe_claim(identity, &privacy));
}

fn rejected(
    token: crate::delivery_events::DecisionToken,
    identity: &ghostr_engine::representation::TransferIdentity,
    commands: &crate::delivery_events::CommandReceiver,
) -> crate::delivery_events::DecisionToken {
    match commands.claim_decision(token, identity, 100) {
        Err(token) => token,
        Ok(_) => panic!("mismatched HEAD identity must be rejected"),
    }
}
