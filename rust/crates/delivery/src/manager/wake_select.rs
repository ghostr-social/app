use super::wake::Wake;
use crate::delivery_events::CommandReceiver;
use crate::delivery_events::DeliveryCommand;
use crate::manager::response_open::ResponseOpenReceiver;
use crate::manager::timeline::TimelineCoordinator;
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
            &mut self.responses,
            &mut self.events,
            &mut self.timelines,
            &mut self.wake_cursor,
        )
        .await
    }
}

pub(crate) async fn wait_for_channel_wake(
    commands: &mut CommandReceiver,
    demand: &mut DemandReceiver,
    responses: &mut ResponseOpenReceiver,
    events: &mut mpsc::UnboundedReceiver<InternalEvent>,
    timelines: &mut TimelineCoordinator,
    cursor: &mut crate::manager::wake_lane::WakeCursor,
) -> Option<Wake> {
    loop {
        if let Some(wake) = ready_wake(commands, demand, responses, events, timelines, cursor) {
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
            Some(response) = responses.recv() => {
                cursor.observe(WakeLane::Response);
                return Some(Wake::Response(Box::new(response)));
            },
            Some(event) = events.recv() => {
                cursor.observe(WakeLane::Internal);
                return Some(Wake::Internal(event));
            },
            Some(result) = timelines.recv() => {
                cursor.observe(WakeLane::Timeline);
                return Some(Wake::Timeline(result));
            },
        }
    }
}

fn ready_wake(
    commands: &mut CommandReceiver,
    demand: &mut DemandReceiver,
    responses: &mut ResponseOpenReceiver,
    events: &mut mpsc::UnboundedReceiver<InternalEvent>,
    timelines: &mut TimelineCoordinator,
    cursor: &mut crate::manager::wake_lane::WakeCursor,
) -> Option<Wake> {
    if let Some(clear) = commands.try_clear() {
        return Some(Wake::Clear(clear));
    }
    if let Some(commands) = commands.try_controls_through_focus() {
        cursor.observe(WakeLane::Control);
        return Some(Wake::Commands(commands));
    }
    let ready = [
        commands.has_control(),
        commands.has_player_preparation(),
        commands.has_playback_presentation(),
        commands.has_candidate(),
        !demand.is_empty(),
        !responses.is_empty(),
        !events.is_empty(),
        timelines.prepare_wake(),
    ];
    match cursor.choose(&ready)? {
        WakeLane::Control => Wake::Command(commands.try_control().expect("ready control lane")),
        WakeLane::PlayerPreparation => Wake::PlayerPreparation(
            commands
                .try_player_preparation()
                .expect("ready player-preparation lane"),
        ),
        WakeLane::PlaybackPresentation => Wake::PlaybackPresentation(
            commands
                .try_playback_presentation()
                .expect("ready playback-presentation lane"),
        ),
        WakeLane::Candidate => Wake::Command(DeliveryCommand::Candidate(
            commands.try_candidate().expect("ready candidate lane"),
        )),
        WakeLane::Demand => Wake::Demand(demand.try_recv().expect("ready demand lane")),
        WakeLane::Response => {
            Wake::Response(Box::new(responses.try_recv().expect("ready response lane")))
        }
        WakeLane::Internal => Wake::Internal(events.try_recv().expect("ready internal lane")),
        WakeLane::Timeline => Wake::Timeline(timelines.take_wake().expect("ready timeline lane")),
    }
    .into()
}
