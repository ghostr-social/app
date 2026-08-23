use super::{retain_preparations, DeliveryState};
use crate::delivery_events::PlayerPreparationReport;

pub(super) fn admit(state: &mut DeliveryState, report: &PlayerPreparationReport) -> bool {
    if report.client_epoch() < state.latest_player_client_epoch {
        return false;
    }
    if report.client_epoch() > state.latest_player_client_epoch {
        state.latest_player_client_epoch = report.client_epoch();
        retain_preparations(
            &mut state.player_preparations,
            &mut state.client_capabilities,
            |_, _| false,
        );
    }
    true
}
