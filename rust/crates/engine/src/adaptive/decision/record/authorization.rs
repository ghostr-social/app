use super::{
    DecisionRecord, CAPABILITY_SCHEMA_VERSION, UNSEALED_WARP_SCHEMA_VERSION, WARP_SCHEMA_VERSION,
};
use crate::adaptive::{DecisionPrivacy, RecordedWarpCommand};
use crate::representation::TransferIdentity;

impl DecisionRecord {
    pub fn authorizes_probe_claim(
        &self,
        identity: &TransferIdentity,
        privacy: &DecisionPrivacy,
    ) -> bool {
        let version = self.schema_version;
        if !matches!(
            version,
            UNSEALED_WARP_SCHEMA_VERSION | WARP_SCHEMA_VERSION | CAPABILITY_SCHEMA_VERSION
        ) || self.chosen_action.is_none()
        {
            return false;
        }
        let command = self
            .warp_decision
            .as_ref()
            .and_then(|decision| decision.selected.as_ref())
            .map(|action| &action.command);
        command.is_some_and(|command| matches_probe(command, identity, privacy))
    }
}

fn matches_probe(
    command: &RecordedWarpCommand,
    identity: &TransferIdentity,
    privacy: &DecisionPrivacy,
) -> bool {
    let RecordedWarpCommand::ProbeHead {
        post_id, source_id, ..
    } = command
    else {
        return false;
    };
    post_id == &privacy.post(identity.post().as_str())
        && source_id == &privacy.source(identity.source().as_str())
}
