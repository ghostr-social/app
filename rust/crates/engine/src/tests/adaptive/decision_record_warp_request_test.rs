use super::decision_record_warp_test_support::{allocation, decision, record};
use crate::adaptive::{
    ActionKind, PlannerCommand, PromotionGrant, RecordedAllocationReason,
    RecordedPreemptionAuthority, RecordedRetrievalRequest, RecordedTransfer, RecordedWarpCommand,
    RecordedWholeBodyContract, RecordedWholeFetchReason, RetrievalRequest, WholeBodyContract,
    WholeFetchReason,
};
use crate::ByteRange;

#[test]
fn transfer_records_preserve_range_promotion_and_whole_body_contracts() {
    let promoted = recorded(RetrievalRequest::FetchRange {
        bytes: ByteRange::new(4, 8),
        promotion: Some(PromotionGrant {
            maximum_bytes: 99,
            valid_until_ms: 123,
        }),
    })
    .request;
    assert!(matches!(
        promoted,
        RecordedRetrievalRequest::FetchRange {
            bytes_start: 4,
            bytes_end: 8,
            promotion: Some(grant)
        } if grant.maximum_bytes == 99 && grant.valid_until_ms == 123
    ));

    let exact = recorded(RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Exact { expected_bytes: 77 },
        reason: WholeFetchReason::DirectCrossover,
    })
    .request;
    let capped = recorded(RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped { maximum_bytes: 88 },
        reason: WholeFetchReason::PlannedCompletion,
    })
    .request;
    let promoted_whole = recorded(RetrievalRequest::FetchWhole {
        contract: WholeBodyContract::Capped { maximum_bytes: 99 },
        reason: WholeFetchReason::PromotedResponse,
    })
    .request;
    assert_eq!(
        exact,
        RecordedRetrievalRequest::FetchWhole {
            contract: RecordedWholeBodyContract::Exact { expected_bytes: 77 },
            reason: RecordedWholeFetchReason::DirectCrossover,
        }
    );
    assert_eq!(
        capped,
        RecordedRetrievalRequest::FetchWhole {
            contract: RecordedWholeBodyContract::Capped { maximum_bytes: 88 },
            reason: RecordedWholeFetchReason::PlannedCompletion,
        }
    );
    assert!(matches!(
        promoted_whole,
        RecordedRetrievalRequest::FetchWhole {
            reason: RecordedWholeFetchReason::PromotedResponse,
            ..
        }
    ));
}

#[test]
fn transfer_records_preserve_the_executor_allocation_contract() {
    let transfer = recorded(RetrievalRequest::FetchRange {
        bytes: ByteRange::new(4, 8),
        promotion: None,
    });
    assert_eq!(transfer.expected_playable_gain_ms, 1_000);
    assert_eq!(transfer.utility.view_probability_bits, 1.0_f64.to_bits());
    assert_eq!(transfer.utility.additional_playable_ms, 1_000);
    assert_eq!(transfer.utility.expected_delivery_ms, 10);
    assert_eq!(transfer.utility.score_bits, 1.0_f64.to_bits());
    assert_eq!(
        transfer.authority,
        RecordedPreemptionAuthority::PlaybackCritical
    );
    assert_eq!(transfer.commitment_until_ms, 1_000);
    assert_eq!(transfer.reason, RecordedAllocationReason::MediaBootstrap);
}

fn recorded(request: RetrievalRequest) -> RecordedTransfer {
    let command = PlannerCommand::Transfer(allocation("https://origin.example/media", request));
    let kind = ActionKind::FetchWhole {
        maximum_bytes: request.reserved_network_bytes(),
    };
    let captured = record(&decision("secret-post", command, kind));
    match captured.warp_decision.unwrap().selected.unwrap().command {
        RecordedWarpCommand::Transfer { transfer } => transfer,
        other => panic!("expected transfer, got {other:?}"),
    }
}
