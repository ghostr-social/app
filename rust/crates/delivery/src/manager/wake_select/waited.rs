use super::WakeSources;
use crate::delivery_events::ClearRequest;
use crate::manager::response_open::ResponseOpenRequest;
use crate::manager::timeline::TimelineResult;
use crate::manager::transfers::InternalEvent;
use crate::manager::wake::Wake;
use crate::manager::wake_lane::{WakeCursor, WakeLane};
use crate::playback_demand::DemandState;

pub(super) enum Resolution {
    Wake(Box<Wake>),
    Retry,
    Closed,
}

pub(super) enum Arrival {
    Control(ControlArrival),
    Worker(Box<WorkerArrival>),
}

pub(super) enum ControlArrival {
    Clear(ClearRequest),
    Mailbox(bool),
    Demand(DemandState),
    Response(Box<ResponseOpenRequest>),
}

pub(super) enum WorkerArrival {
    Internal(InternalEvent),
    Invalidation(bool),
    Timeline(TimelineResult),
    Interval,
}

pub(super) async fn wait(
    sources: &mut WakeSources<'_>,
    interval: &mut tokio::time::Interval,
) -> Arrival {
    let control = wait_control(sources.commands, sources.demand, sources.responses);
    let worker = wait_worker(
        sources.events,
        sources.invalidations,
        sources.timelines,
        interval,
    );
    tokio::select! {
        biased;
        arrival = control => Arrival::Control(arrival),
        arrival = worker => Arrival::Worker(Box::new(arrival)),
    }
}

async fn wait_control(
    commands: &mut crate::delivery_events::CommandReceiver,
    demand: &mut crate::playback_demand::DemandReceiver,
    responses: &mut crate::manager::response_open::ResponseOpenReceiver,
) -> ControlArrival {
    let (mailbox, clears) = commands.receivers();
    tokio::select! {
        biased;
        Some(clear) = clears.recv() => ControlArrival::Clear(clear),
        changed = mailbox.changed() => ControlArrival::Mailbox(changed),
        Some(signal) = demand.recv() => ControlArrival::Demand(signal),
        Some(response) = responses.recv() => ControlArrival::Response(Box::new(response)),
    }
}

async fn wait_worker(
    events: &mut tokio::sync::mpsc::UnboundedReceiver<InternalEvent>,
    invalidations: &mut tokio::sync::watch::Receiver<u64>,
    timelines: &mut crate::manager::timeline::TimelineCoordinator,
    interval: &mut tokio::time::Interval,
) -> WorkerArrival {
    tokio::select! {
        biased;
        Some(event) = events.recv() => WorkerArrival::Internal(event),
        changed = invalidations.changed() => WorkerArrival::Invalidation(changed.is_ok()),
        Some(result) = timelines.recv() => WorkerArrival::Timeline(result),
        _ = interval.tick() => WorkerArrival::Interval,
    }
}

impl Arrival {
    pub(super) fn resolve(self, cursor: &mut WakeCursor) -> Resolution {
        match self {
            Self::Control(arrival) => arrival.resolve(cursor),
            Self::Worker(arrival) => arrival.resolve(cursor),
        }
    }
}

impl ControlArrival {
    fn resolve(self, cursor: &mut WakeCursor) -> Resolution {
        match self {
            Self::Clear(clear) => Resolution::Wake(Box::new(Wake::Clear(clear))),
            Self::Mailbox(true) => Resolution::Retry,
            Self::Mailbox(false) => Resolution::Closed,
            Self::Demand(signal) => observed(cursor, WakeLane::Demand, Wake::Demand(signal)),
            Self::Response(response) => {
                observed(cursor, WakeLane::Response, Wake::Response(response))
            }
        }
    }
}

impl WorkerArrival {
    fn resolve(self, cursor: &mut WakeCursor) -> Resolution {
        match self {
            Self::Internal(event) => observed(cursor, WakeLane::Internal, Wake::Internal(event)),
            Self::Invalidation(true) => observed(
                cursor,
                WakeLane::SegmentedInvalidation,
                Wake::SegmentedInvalidated,
            ),
            Self::Invalidation(false) => Resolution::Retry,
            Self::Timeline(result) => observed(cursor, WakeLane::Timeline, Wake::Timeline(result)),
            Self::Interval => Resolution::Wake(Box::new(Wake::ControlInterval)),
        }
    }
}

fn observed(cursor: &mut WakeCursor, lane: WakeLane, wake: Wake) -> Resolution {
    cursor.observe(lane);
    Resolution::Wake(Box::new(wake))
}
