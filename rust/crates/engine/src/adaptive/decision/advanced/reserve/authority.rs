use super::{DecisionPrivacy, RecordedReserveAuthorityOccupancy, ReserveAuthorityOccupancy};
use crate::RequestAuthority;

pub(super) fn capture(
    value: &ReserveAuthorityOccupancy,
    privacy: &DecisionPrivacy,
) -> RecordedReserveAuthorityOccupancy {
    RecordedReserveAuthorityOccupancy {
        authority_id: privacy.authority(value.authority.as_str()),
        occupied_request_slots: u64::try_from(value.occupied_request_slots)
            .expect("request occupancy fits the schema-v2 counter"),
        request_width: value.request_width,
    }
}

pub(super) fn sorted(
    mut values: Vec<RecordedReserveAuthorityOccupancy>,
) -> Vec<RecordedReserveAuthorityOccupancy> {
    values.sort_by(|left, right| left.authority_id.cmp(&right.authority_id));
    values
}

pub(super) fn restore(
    value: &RecordedReserveAuthorityOccupancy,
) -> Option<ReserveAuthorityOccupancy> {
    Some(ReserveAuthorityOccupancy {
        authority: RequestAuthority::from_url(&value.authority_id)?,
        occupied_request_slots: usize::try_from(value.occupied_request_slots).ok()?,
        request_width: value.request_width,
    })
}
