use super::wake::Wake;
use crate::delivery_events::CommandReceiver;
use crate::manager::response_open::ResponseOpenReceiver;
use crate::manager::timeline::TimelineCoordinator;
use crate::manager::transfers::InternalEvent;
use crate::manager::wake_lane::WakeCursor;
use crate::manager::DeliveryWorker;
use crate::playback_demand::DemandReceiver;
use tokio::sync::{mpsc, watch};

mod ready;
mod waited;

pub(crate) struct WakeSources<'a> {
    pub(crate) commands: &'a mut CommandReceiver,
    pub(crate) demand: &'a mut DemandReceiver,
    pub(crate) responses: &'a mut ResponseOpenReceiver,
    pub(crate) events: &'a mut mpsc::UnboundedReceiver<InternalEvent>,
    pub(crate) invalidations: &'a mut watch::Receiver<u64>,
    pub(crate) timelines: &'a mut TimelineCoordinator,
}

impl DeliveryWorker {
    pub(super) async fn next_wake(&mut self) -> Option<Wake> {
        let mut sources = WakeSources {
            commands: &mut self.commands,
            demand: &mut self.demand,
            responses: &mut self.responses,
            events: &mut self.events,
            invalidations: &mut self.segmented_invalidations,
            timelines: &mut self.timelines,
        };
        wait_for_channel_wake(
            &mut sources,
            &mut self.control_interval,
            &mut self.wake_cursor,
        )
        .await
    }
}

pub(crate) async fn wait_for_channel_wake(
    sources: &mut WakeSources<'_>,
    control_interval: &mut tokio::time::Interval,
    cursor: &mut WakeCursor,
) -> Option<Wake> {
    loop {
        if let Some(wake) = ready::take(sources, cursor) {
            return Some(wake);
        }
        match waited::wait(sources, control_interval)
            .await
            .resolve(cursor)
        {
            waited::Resolution::Wake(wake) => return Some(*wake),
            waited::Resolution::Retry => {}
            waited::Resolution::Closed => return None,
        }
    }
}
