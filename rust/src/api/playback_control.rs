use crate::api::delivery::playback_mapping::{playback_presentation, playback_update};
use crate::api::playback_types::{
    FfiPlaybackAdmissionSnapshot, FfiPlaybackObservation, FfiPlaybackPresentation,
};
use crate::api::runtime::registry;
use flutter_rust_bridge::frb;
use ghostr_delivery::delivery_events::PlaybackPresentationIngress;

/// Reports one ordered playback snapshot to the single delivery manager.
#[frb]
pub async fn ffi_report_playback(input: FfiPlaybackObservation) -> anyhow::Result<()> {
    let update = playback_update(input)?;
    registry::engine()?
        .gateway
        .delivery()
        .report_playback(update);
    Ok(())
}

/// Reports the one user-visible frame separately from coalesced phase samples.
#[frb]
pub async fn ffi_report_playback_presentation(
    input: FfiPlaybackPresentation,
) -> anyhow::Result<()> {
    let event = playback_presentation(input)?;
    let admission = registry::engine()?
        .gateway
        .delivery()
        .report_playback_presentation(event);
    match admission {
        PlaybackPresentationIngress::Accepted | PlaybackPresentationIngress::Stale => Ok(()),
        PlaybackPresentationIngress::Saturated => {
            anyhow::bail!("playback presentation mailbox is saturated")
        }
        PlaybackPresentationIngress::Closed => {
            anyhow::bail!("delivery manager is unavailable")
        }
    }
}

/// Returns process-lifetime playback decisions and latest accepted identity.
#[frb]
pub async fn ffi_playback_admission_snapshot() -> anyhow::Result<FfiPlaybackAdmissionSnapshot> {
    let snapshot = registry::engine()?
        .gateway
        .delivery()
        .playback_admission_snapshot();
    Ok(snapshot.into())
}
