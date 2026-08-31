use super::{range, RangeSnapshot};
use ghostr_engine::adaptive::{
    HlsBootstrapStage, NextReserveEvidence, NextReserveInfeasibility,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(super) enum NextReserveSnapshot {
    NotApplicable,
    Ready {
        post_id: String,
    },
    Structural {
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
    HlsReady {
        post_id: String,
    },
    HlsStructural {
        post_id: String,
    },
    HlsInFlight {
        post_id: String,
        stage: &'static str,
    },
    HlsPending {
        post_id: String,
        stage: &'static str,
    },
}

pub(super) fn snapshot(value: &NextReserveEvidence) -> NextReserveSnapshot {
    match value {
        NextReserveEvidence::NotApplicable => NextReserveSnapshot::NotApplicable,
        NextReserveEvidence::Ready { post, .. } => NextReserveSnapshot::Ready {
            post_id: post.as_str().to_owned(),
        },
        NextReserveEvidence::Structural { post, .. } => NextReserveSnapshot::Structural {
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
        NextReserveEvidence::HlsReady { post } => NextReserveSnapshot::HlsReady {
            post_id: post.as_str().to_owned(),
        },
        NextReserveEvidence::HlsStructural { post } => NextReserveSnapshot::HlsStructural {
            post_id: post.as_str().to_owned(),
        },
        NextReserveEvidence::HlsInFlight { post, stage } => NextReserveSnapshot::HlsInFlight {
            post_id: post.as_str().to_owned(),
            stage: hls_stage(*stage),
        },
        NextReserveEvidence::HlsPending { post, stage } => NextReserveSnapshot::HlsPending {
            post_id: post.as_str().to_owned(),
            stage: hls_stage(*stage),
        },
    }
}

pub(super) const fn hls_stage(value: HlsBootstrapStage) -> &'static str {
    match value {
        HlsBootstrapStage::RootManifest => "root_manifest",
        HlsBootstrapStage::ChildPlaylist => "child_playlist",
        HlsBootstrapStage::Initialization => "initialization",
        HlsBootstrapStage::FirstSegment => "first_segment",
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
