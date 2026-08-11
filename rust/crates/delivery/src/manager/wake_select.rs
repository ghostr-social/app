use super::wake::Wake;
use crate::delivery_events::CommandReceiver;
use crate::delivery_events::DeliveryCommand;
use crate::manager::transfers::InternalEvent;
use crate::manager::wake_lane::WakeLane;
use crate::manager::DeliveryWorker;
use crate::playback_demand::DemandReceiver;
use tokio::sync::mpsc;

impl DeliveryWorker {
    pub(super) async fn next_wake(&mut self) -> Option<Wake> {
        wait_for_channel_wake(
            &mut self.commands,
            &mut self.demand,
            &mut self.events,
            &mut self.wake_cursor,
        )
        .await
    }
}

pub(crate) async fn wait_for_channel_wake(
    commands: &mut CommandReceiver,
    demand: &mut DemandReceiver,
    events: &mut mpsc::UnboundedReceiver<InternalEvent>,
    cursor: &mut crate::manager::wake_lane::WakeCursor,
) -> Option<Wake> {
    loop {
        if let Some(wake) = ready_wake(commands, demand, events, cursor) {
            return Some(wake);
        }
        let (mailbox, clears) = commands.receivers();
        tokio::select! {
            biased;
            Some(clear) = clears.recv() => return Some(Wake::Clear(clear)),
            changed = mailbox.changed() => if !changed { return None },
            Some(signal) = demand.recv() => {
                cursor.observe(WakeLane::Demand);
                return Some(Wake::Demand(signal));
            },
            Some(event) = events.recv() => {
                cursor.observe(WakeLane::Internal);
                return Some(Wake::Internal(event));
            },
        }
    }
}

fn ready_wake(
    commands: &mut CommandReceiver,
    demand: &mut DemandReceiver,
    events: &mut mpsc::UnboundedReceiver<InternalEvent>,
    cursor: &mut crate::manager::wake_lane::WakeCursor,
) -> Option<Wake> {
    if let Some(clear) = commands.try_clear() {
        return Some(Wake::Clear(clear));
    }
    let ready = [
        commands.has_control(),
        commands.has_candidate(),
        !demand.is_empty(),
        !events.is_empty(),
    ];
    match cursor.choose(&ready)? {
        WakeLane::Control => Wake::Command(commands.try_control().expect("ready control lane")),
        WakeLane::Candidate => Wake::Command(DeliveryCommand::Candidate(
            commands.try_candidate().expect("ready candidate lane"),
        )),
        WakeLane::Demand => Wake::Demand(demand.try_recv().expect("ready demand lane")),
        WakeLane::Internal => Wake::Internal(events.try_recv().expect("ready internal lane")),
    }
    .into()
}
