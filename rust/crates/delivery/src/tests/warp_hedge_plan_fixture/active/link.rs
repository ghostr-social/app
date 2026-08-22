use super::{attempt, insert, Registration};
use crate::manager::inflight::{ChunkAttempt, InFlightChunks};
use crate::manager::state::DeliveryState;
use crate::tests::warp_hedge_plan_fixture::{HedgeCase, ALTERNATE};
use ghostr_engine::ChunkId;

pub(super) struct Input<'a> {
    pub(super) state: &'a DeliveryState,
    pub(super) chunk: ChunkId,
    pub(super) primary: &'a ChunkAttempt,
    pub(super) case: HedgeCase,
}

pub(super) fn alternate(active: &mut InFlightChunks, input: Input<'_>) {
    if !input.case.linked() {
        return;
    }
    let range = input.chunk.range;
    let alternate = attempt(active, input.state, input.chunk, ALTERNATE);
    insert(
        active,
        Registration {
            attempt: &alternate,
            source: ALTERNATE,
            range,
            launched_at_ms: 4_900,
        },
    );
    assert!(active.link_hedge(input.primary.id(), alternate.id()));
}
