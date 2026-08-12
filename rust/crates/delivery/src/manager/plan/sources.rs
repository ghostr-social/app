use super::{startup_seconds, PlanInputs, StartupContext};
use crate::manager::state::DeliveryState;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::inventory_controller::Mode;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;
use std::collections::HashMap;

/// Best live representation-fenced source per catalogued window post.
#[derive(Default)]
pub(super) struct SourceChoices {
    pub(super) urls: HashMap<PostId, String>,
    pub(super) identities: HashMap<PostId, TransferIdentity>,
    pub(super) head_seconds: HashMap<PostId, u64>,
}

pub(super) fn source_choices(state: &DeliveryState, inputs: &PlanInputs<'_>) -> SourceChoices {
    let mut choices = SourceChoices::default();
    for post in state.protected_posts() {
        if inputs.retry.is_cooling(&post) {
            continue;
        }
        let Some(choice) = source_choice(state, inputs, &post) else {
            continue;
        };
        choices.urls.insert(post.clone(), choice.url);
        choices.identities.insert(post.clone(), choice.identity);
        choices.head_seconds.insert(post, choice.head_seconds);
    }
    choices
}

pub(super) fn host_factor(
    urls: &HashMap<PostId, String>,
    post: &PostId,
    stats: &HostStats,
    mode: Mode,
) -> f64 {
    urls.get(post)
        .and_then(|url| host_of(url))
        .map(|host| stats.host_factor(&host, mode))
        .unwrap_or(1.0)
}

struct SourceChoice {
    url: String,
    identity: TransferIdentity,
    head_seconds: u64,
}

fn source_choice(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    post: &PostId,
) -> Option<SourceChoice> {
    let entry = state.catalog().lookup(post)?;
    entry.total_bytes()?;
    let live = inputs.retry.live_urls(post, &entry.meta.urls);
    let url = inputs
        .stats
        .best_source(&live, Mode::Hunger)
        .first()?
        .clone();
    let host = host_of(&url)?;
    let identity = state.catalog().transfer_identity(post, &url)?;
    let context = StartupContext::new(
        state.catalog().estimated_bitrate(post, state.params()),
        inputs.observed_at_ms,
        state.params().head_seconds,
    );
    Some(SourceChoice {
        url,
        identity,
        head_seconds: startup_seconds(inputs.stats, &host, context),
    })
}
