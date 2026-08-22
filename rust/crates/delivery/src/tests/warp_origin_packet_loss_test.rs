use crate::tests::adaptive_plan_support::plan_with_packet_loss;

#[test]
fn network_packet_loss_reaches_every_candidate_origin() {
    let work = plan_with_packet_loss(4_321);
    let snapshot = work.snapshot.expect("playability snapshot");
    let origins = snapshot
        .candidates
        .iter()
        .flat_map(|candidate| &candidate.origins)
        .collect::<Vec<_>>();

    assert!(!origins.is_empty());
    assert!(origins.iter().all(|origin| origin.packet_loss_bps == 4_321));
}
