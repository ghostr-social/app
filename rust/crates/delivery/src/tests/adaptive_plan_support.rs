use crate::manager::inflight::ActiveRange;
use crate::manager::plan::PlannedWork;
use crate::tests::adaptive_plan_fixture::{source, state};
use crate::tests::adaptive_plan_runner::{run, PlanScenario};
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::{ByteRange, ChunkId, PostId};
use std::collections::HashMap;

pub(super) fn plan(buffer_ms: u64, bytes_per_second: u64, storage: StorageSnapshot) -> PlannedWork {
    run(PlanScenario {
        state: state(),
        buffer_ms,
        bytes_per_second,
        storage,
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 4,
    })
}

pub(super) fn plan_with_capacity(
    buffer_ms: u64,
    bytes_per_second: u64,
    storage: StorageSnapshot,
    connection_capacity: usize,
) -> PlannedWork {
    run(PlanScenario {
        state: state(),
        buffer_ms,
        bytes_per_second,
        storage,
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity,
    })
}

pub(super) fn plan_with_active(range: ByteRange, committed_until_ms: u64) -> PlannedWork {
    let state = state();
    let post = PostId::new("p1");
    let identity = state
        .catalog()
        .transfer_identity(&post, &source(1))
        .unwrap();
    let active = ActiveRange::new(ChunkId { post, range }, identity, committed_until_ms);
    run(PlanScenario {
        state,
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[active],
        connection_capacity: 4,
    })
}

pub(super) fn plan_existing(state: crate::manager::state::DeliveryState) -> PlannedWork {
    run(PlanScenario {
        state,
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 4,
    })
}

pub(super) fn plan_with_present(
    storage: StorageSnapshot,
    present: HashMap<PostId, Vec<ByteRange>>,
) -> PlannedWork {
    run(PlanScenario {
        state: state(),
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage,
        present,
        packet_loss_bps: 0,
        in_flight: &[],
        connection_capacity: 4,
    })
}

pub(super) fn plan_with_packet_loss(packet_loss_bps: u16) -> PlannedWork {
    run(PlanScenario {
        state: state(),
        buffer_ms: 20_000,
        bytes_per_second: 4_000_000,
        storage: StorageSnapshot::new(2_000_000_000, 0),
        present: HashMap::new(),
        packet_loss_bps,
        in_flight: &[],
        connection_capacity: 4,
    })
}
