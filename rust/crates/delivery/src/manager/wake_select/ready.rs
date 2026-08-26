use super::WakeSources;
use crate::delivery_events::DeliveryCommand;
use crate::manager::wake::Wake;
use crate::manager::wake_lane::{WakeCursor, WakeLane};

pub(super) fn take(sources: &mut WakeSources<'_>, cursor: &mut WakeCursor) -> Option<Wake> {
    priority(sources, cursor).or_else(|| fair(sources, cursor))
}

fn priority(sources: &mut WakeSources<'_>, cursor: &mut WakeCursor) -> Option<Wake> {
    if let Some(clear) = sources.commands.try_clear() {
        return Some(Wake::Clear(clear));
    }
    if let Some(commands) = sources.commands.try_controls_through_focus() {
        cursor.observe(WakeLane::Control);
        return Some(Wake::Commands(commands));
    }
    None
}

fn fair(sources: &mut WakeSources<'_>, cursor: &mut WakeCursor) -> Option<Wake> {
    let ready = [
        sources.commands.has_control(),
        sources.commands.has_player_preparation(),
        sources.commands.has_playback_presentation(),
        sources.commands.has_candidate(),
        !sources.demand.is_empty(),
        !sources.responses.is_empty(),
        !sources.events.is_empty(),
        sources.invalidations.has_changed().unwrap_or(false),
        sources.timelines.prepare_wake(),
    ];
    Some(take_lane(sources, cursor.choose(&ready)?))
}

fn take_lane(sources: &mut WakeSources<'_>, lane: WakeLane) -> Wake {
    match lane {
        WakeLane::Control
        | WakeLane::PlayerPreparation
        | WakeLane::PlaybackPresentation
        | WakeLane::Candidate => command_lane(sources, lane),
        WakeLane::Demand | WakeLane::Response | WakeLane::Internal => io_lane(sources, lane),
        WakeLane::SegmentedInvalidation | WakeLane::Timeline => system_lane(sources, lane),
    }
}

fn command_lane(sources: &mut WakeSources<'_>, lane: WakeLane) -> Wake {
    match lane {
        WakeLane::Control => Wake::Command(sources.commands.try_control().expect("control lane")),
        WakeLane::PlayerPreparation => Wake::PlayerPreparation(
            sources
                .commands
                .try_player_preparation_envelope()
                .expect("player-preparation lane"),
        ),
        WakeLane::PlaybackPresentation => Wake::PlaybackPresentation(
            sources
                .commands
                .try_playback_presentation()
                .expect("playback-presentation lane"),
        ),
        WakeLane::Candidate => Wake::Command(DeliveryCommand::Candidate(
            sources.commands.try_candidate().expect("candidate lane"),
        )),
        _ => unreachable!("command lane group is exhaustive"),
    }
}

fn io_lane(sources: &mut WakeSources<'_>, lane: WakeLane) -> Wake {
    match lane {
        WakeLane::Demand => Wake::Demand(sources.demand.try_recv().expect("demand lane")),
        WakeLane::Response => Wake::Response(Box::new(
            sources.responses.try_recv().expect("response lane"),
        )),
        WakeLane::Internal => Wake::Internal(sources.events.try_recv().expect("internal lane")),
        _ => unreachable!("IO lane group is exhaustive"),
    }
}

fn system_lane(sources: &mut WakeSources<'_>, lane: WakeLane) -> Wake {
    match lane {
        WakeLane::SegmentedInvalidation => {
            sources.invalidations.borrow_and_update();
            Wake::SegmentedInvalidated
        }
        WakeLane::Timeline => Wake::Timeline(sources.timelines.take_wake().expect("timeline lane")),
        _ => unreachable!("system lane group is exhaustive"),
    }
}
