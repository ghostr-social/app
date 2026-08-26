use crate::manager::reconcile_warp::axiom_test_support::directive_for;
use crate::manager::reconcile_warp::WarpDirective;
use crate::tests::support::planned_transfer;
use ghostr_engine::adaptive::{
    Allocation, AllocationReason, CandidateUtility, PlannerCommand, PreemptionAuthority,
};
use ghostr_engine::ActionId;

#[test]
fn selected_hedge_retains_its_primary_and_links_the_exact_alternate() {
    let transfer = planned_transfer(
        "video",
        "alternate.example",
        PreemptionAuthority::Transition,
    );
    let command = PlannerCommand::Hedge {
        primary: ActionId::new(7),
        transfer: allocation(&transfer),
    };

    assert_eq!(
        directive_for(Some(&command), core::slice::from_ref(&transfer)),
        WarpDirective::Hedge {
            primary: ActionId::new(7),
            alternate: transfer.id(),
        }
    );
}

fn allocation(transfer: &crate::manager::plan::PlannedTransfer) -> Allocation {
    Allocation {
        post: transfer.request.chunk.post.clone(),
        request: transfer.retrieval,
        source: transfer.url.clone(),
        expected_playable_gain_ms: 0,
        utility: CandidateUtility {
            view_probability: 1.0,
            additional_playable_ms: 0,
            expected_delivery_ms: 1,
            score: 1.0,
        },
        authority: transfer.request.authority,
        commitment_until_ms: transfer.commitment_until_ms,
        reason: AllocationReason::CurrentStallPrevention,
    }
}
