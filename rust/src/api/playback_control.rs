use crate::api::delivery::playback_mapping::playback_update;
use crate::api::playback_types::FfiPlaybackObservation;
use crate::api::runtime::registry;
use flutter_rust_bridge::frb;

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
