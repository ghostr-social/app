use super::{range, RangeSnapshot};
use ghostr_engine::adaptive::{NextReserveEvidence, NextReserveInfeasibility};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(super) enum NextReserveSnapshot {
    NotApplicable,
    Ready {
        post_id: String,
    },
    InFlight {
        post_id: String,
    },
    Granted {
        post_id: String,
        range: RangeSnapshot,
    },
    Infeasible {
        post_id: String,
        reason: &'static str,
    },
}

pub(super) fn snapshot(value: &NextReserveEvidence) -> NextReserveSnapshot {
    match value {
        NextReserveEvidence::NotApplicable => NextReserveSnapshot::NotApplicable,
        NextReserveEvidence::Ready { post } => NextReserveSnapshot::Ready {
            post_id: post.as_str().to_owned(),
        },
        NextReserveEvidence::InFlight { post } => NextReserveSnapshot::InFlight {
            post_id: post.as_str().to_owned(),
        },
        NextReserveEvidence::Granted { post, range: span } => NextReserveSnapshot::Granted {
            post_id: post.as_str().to_owned(),
            range: range(*span),
        },
        NextReserveEvidence::Infeasible { post, reason } => NextReserveSnapshot::Infeasible {
            post_id: post.as_str().to_owned(),
            reason: infeasibility(*reason),
        },
    }
}

pub(super) fn infeasibility(value: NextReserveInfeasibility) -> &'static str {
    match value {
        NextReserveInfeasibility::CurrentUnprotected => "current_unprotected",
        NextReserveInfeasibility::NoLiveOrigin => "no_live_origin",
        NextReserveInfeasibility::PolicyDenied => "policy_denied",
        NextReserveInfeasibility::NoTransferBudget => "no_transfer_budget",
        NextReserveInfeasibility::NoStorageCapacity => "no_storage_capacity",
    }
}
