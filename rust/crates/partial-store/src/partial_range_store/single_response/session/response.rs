use super::super::{ResponseOwner, SingleResponseState};
use crate::partial_range_manifest::RangeManifest;
use ghostr_engine::representation::TransferIdentity;

#[derive(Clone)]
enum SessionPhase {
    Active(ResponseOwner),
    Complete,
}

#[derive(Clone)]
pub(in crate::partial_range_store) struct SessionResponse {
    identity: TransferIdentity,
    manifest: RangeManifest,
    phase: SessionPhase,
}

impl SessionResponse {
    pub(super) fn complete(identity: TransferIdentity, manifest: RangeManifest) -> Self {
        Self {
            identity,
            manifest,
            phase: SessionPhase::Complete,
        }
    }

    pub(in crate::partial_range_store::single_response) fn active(
        state: &SingleResponseState,
    ) -> Option<Self> {
        state.owner.is_active().then_some(())?;
        Some(Self {
            identity: state.identity.clone(),
            manifest: state.readable_prefix()?.clone(),
            phase: SessionPhase::Active(state.owner.clone()),
        })
    }

    pub(in crate::partial_range_store) fn identity(&self) -> &TransferIdentity {
        &self.identity
    }

    pub(in crate::partial_range_store) fn manifest(&self) -> &RangeManifest {
        &self.manifest
    }

    pub(in crate::partial_range_store) fn is_complete(&self) -> bool {
        matches!(self.phase, SessionPhase::Complete)
    }

    pub(in crate::partial_range_store) fn is_readable(&self) -> bool {
        match &self.phase {
            SessionPhase::Active(owner) => owner.is_active(),
            SessionPhase::Complete => true,
        }
    }

    pub(super) fn bytes(&self) -> u64 {
        self.manifest.covered_bytes()
    }
}
