use super::super::{RecordedAuthorityOccupancy, RecordedWarpSearchInput};
use crate::adaptive::{RecordedReserveAuthorityOccupancy, RecordedWarpReserve};
use crate::RequestAuthority;

pub(super) fn matches(input: &RecordedWarpSearchInput, reserve: &RecordedWarpReserve) -> bool {
    unique(reserve)
        && protected_recorded(input, reserve)
        && reserve
            .authority_occupancy
            .iter()
            .all(|expected| reserve_entry_matches(input, expected))
        && input.budget.origins.iter().all(|actual| {
            reserve
                .authority_occupancy
                .iter()
                .any(|expected| occupied_matches(actual, expected))
        })
}

fn protected_recorded(input: &RecordedWarpSearchInput, reserve: &RecordedWarpReserve) -> bool {
    input.actions.iter().all(|action| {
        !input
            .budget
            .pending_rescue_action_ids
            .contains(&action.planner_action_id)
            || action
                .request_source_id
                .as_deref()
                .is_none_or(|source| reserve_has(reserve, source))
    })
}

fn reserve_has(reserve: &RecordedWarpReserve, source: &str) -> bool {
    let source = RequestAuthority::from_url(source);
    reserve
        .authority_occupancy
        .iter()
        .any(|item| RequestAuthority::from_url(&item.authority_id) == source)
}

fn reserve_entry_matches(
    input: &RecordedWarpSearchInput,
    expected: &RecordedReserveAuthorityOccupancy,
) -> bool {
    if u64::from(expected.request_width) != input.budget.per_origin_requests {
        return false;
    }
    if expected.occupied_request_slots == 0 {
        return protected(input, &expected.authority_id);
    }
    input
        .budget
        .origins
        .iter()
        .any(|actual| occupied_matches(actual, expected))
}

fn unique(reserve: &RecordedWarpReserve) -> bool {
    reserve
        .authority_occupancy
        .iter()
        .enumerate()
        .all(|(index, item)| {
            reserve.authority_occupancy[..index]
                .iter()
                .all(|prior| prior.authority_id != item.authority_id)
        })
}

fn occupied_matches(
    actual: &RecordedAuthorityOccupancy,
    expected: &RecordedReserveAuthorityOccupancy,
) -> bool {
    actual.source_id == expected.authority_id && actual.requests == expected.occupied_request_slots
}

fn protected(input: &RecordedWarpSearchInput, authority: &str) -> bool {
    let expected = RequestAuthority::from_url(authority);
    input.actions.iter().any(|action| {
        input
            .budget
            .pending_rescue_action_ids
            .contains(&action.planner_action_id)
            && action
                .request_source_id
                .as_deref()
                .and_then(RequestAuthority::from_url)
                == expected
    })
}
