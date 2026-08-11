use super::wake::Wake;
use crate::delivery_events::DeliveryCommand;
use crate::manager::wake_lane::{WakeLane, WAKE_LANES};
use crate::manager::DeliveryWorker;

impl DeliveryWorker {
    pub(super) async fn next_wake(&mut self) -> Option<Wake> {
        loop {
            if let Some(wake) = self.ready_wake() {
                return Some(wake);
            }
            let (commands, clears) = self.commands.receivers();
            tokio::select! {
                biased;
                Some(clear) = clears.recv() => return Some(Wake::Clear(clear)),
                changed = commands.changed() => if !changed { return None },
                Some(signal) = self.demand.recv() => {
                    self.wake_cursor.observe(WakeLane::Demand);
                    return Some(Wake::Demand(signal));
                },
                Some(event) = self.events.recv() => {
                    self.wake_cursor.observe(WakeLane::Internal);
                    return Some(Wake::Internal(event));
                },
            }
        }
    }

    fn ready_wake(&mut self) -> Option<Wake> {
        if let Some(clear) = self.commands.try_clear() {
            return Some(Wake::Clear(clear));
        }
        let lane = self.wake_cursor.choose(&self.ready_lanes())?;
        Some(self.take_lane(lane))
    }

    fn ready_lanes(&self) -> [bool; WAKE_LANES] {
        [
            self.commands.has_control(),
            self.commands.has_candidate(),
            !self.demand.is_empty(),
            !self.events.is_empty(),
        ]
    }

    fn take_lane(&mut self, lane: WakeLane) -> Wake {
        match lane {
            WakeLane::Control => Wake::Command(self.control_command()),
            WakeLane::Candidate => Wake::Command(DeliveryCommand::Candidate(
                self.commands.try_candidate().expect("ready candidate lane"),
            )),
            WakeLane::Demand => Wake::Demand(self.demand.try_recv().expect("ready demand lane")),
            WakeLane::Internal => {
                Wake::Internal(self.events.try_recv().expect("ready internal lane"))
            }
        }
    }

    fn control_command(&mut self) -> DeliveryCommand {
        self.commands.try_control().expect("ready control lane")
    }
}
