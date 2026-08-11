//! Runtime rendition choice from the manager's playback and network evidence.

use crate::manager::state::DeliveryState;
use crate::manager::DeliveryWorker;
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::inventory_controller::Mode;
use ghostr_engine::playback::{
    AdaptiveBufferPolicy, BufferTarget, MediaConsumption, NetworkConditions, PlaybackObservation,
};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use std::time::Duration;

struct SelectionEvidence {
    post: PostId,
    network: NetworkConditions,
    observation: PlaybackObservation,
    target: BufferTarget,
}

pub(crate) fn select_rendition(
    state: &mut DeliveryState,
    stats: &HostStats,
    observed_at_ms: u64,
) -> Option<RepresentationBinding> {
    let evidence = evidence(state, stats, observed_at_ms)?;
    state.catalog_mut().select_rendition(
        &evidence.post,
        evidence.network,
        evidence.observation,
        evidence.target,
    )
}

impl DeliveryWorker {
    pub(super) async fn select_playback_rendition(&mut self, observed_at_ms: u64) {
        let Some(binding) = select_rendition(&mut self.state, self.keeper.stats(), observed_at_ms)
        else {
            return;
        };
        self.cooldown_timers.cancel(binding.post());
        prepare_rendition_switch(&mut self.state, &mut self.probes, &mut self.retry, binding);
        self.bind_representations().await;
    }
}

pub(crate) fn prepare_rendition_switch(
    state: &mut DeliveryState,
    probes: &mut MetadataProbePool,
    retry: &mut crate::manager::retry::RetryBook,
    binding: RepresentationBinding,
) {
    probes.representation_changed(binding.post());
    retry.representation_changed(binding.post());
    state.queue_representation(binding);
}

fn evidence(
    state: &DeliveryState,
    stats: &HostStats,
    observed_at_ms: u64,
) -> Option<SelectionEvidence> {
    let session = state.playback().session()?;
    let post = session.post().clone();
    let observation = state.playback().observation()?;
    let host = active_host(state, stats, &post)?;
    let network = network(stats, &host, observed_at_ms)?;
    let bitrate = state.catalog().estimated_bitrate(&post, state.params());
    let media = MediaConsumption::new(bitrate, observation.playback_rate_milli());
    let target = AdaptiveBufferPolicy::default().target(network, media);
    Some(SelectionEvidence {
        post,
        network,
        observation,
        target,
    })
}

fn active_host(state: &DeliveryState, stats: &HostStats, post: &PostId) -> Option<String> {
    let urls = &state.catalog().lookup(post)?.meta.urls;
    let source = stats.best_source(urls, Mode::Hunger).into_iter().next()?;
    host_of(&source)
}

fn network(stats: &HostStats, host: &str, observed_at_ms: u64) -> Option<NetworkConditions> {
    let estimate = stats
        .host_throughput(host)
        .or_else(|| stats.overall_throughput())?;
    let ttfb = stats
        .expected_ttfb(host)
        .or_else(|| stats.overall_ttfb())
        .unwrap_or(Duration::from_millis(250));
    Some(NetworkConditions::from_estimate(
        estimate,
        ttfb,
        observed_at_ms,
    ))
}
