//! Per-post readiness and progress events for one watcher pass.

use crate::api::delivery::causal_eta::with_causal_eta;
use crate::api::delivery::snapshot_view::{self, SnapshotViewContext};
use crate::api::delivery::snapshots::{
    compute_snapshot, error_event, event_for, hls_snapshot, SnapshotInput,
};
use crate::api::delivery_events_stream::EventOut;
use crate::engine::{DeliveryKind, EngineParams, PostId, VideoMeta};
use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::collections::HashMap;

pub(crate) use crate::api::delivery::snapshots::DeliverySnapshot;

pub(crate) struct Pass<'a> {
    pub(crate) store: &'a PartialRangeStore,
    pub(crate) segmented: &'a SegmentedCache,
    pub(crate) cache: &'a CacheRegistry,
    pub(crate) capabilities: &'a ProgressiveCapabilities,
    pub(crate) params: &'a EngineParams,
    pub(crate) delivery: Option<&'a DeliveryHandle>,
    pub(crate) focus_generation: Option<u64>,
    pub(crate) emitted: &'a mut HashMap<String, DeliverySnapshot>,
}

pub(crate) async fn emit_post(
    out: &impl EventOut,
    pass: &mut Pass<'_>,
    id: &str,
    meta: &VideoMeta,
) -> bool {
    if meta.delivery == DeliveryKind::Hls {
        return emit_hls(out, pass, id);
    }
    let view = match snapshot_view::capture(
        SnapshotViewContext {
            store: pass.store,
            cache: pass.cache,
            capabilities: pass.capabilities,
        },
        id,
        meta,
    )
    .await
    {
        Ok(view) => view,
        Err(error) => return out.send(error_event(id, error.to_string())),
    };
    let meta = view.meta.as_ref().unwrap_or(meta);
    let post = PostId::new(id);
    let input = SnapshotInput {
        meta,
        ranges: &view.ranges,
        stored_total: view.stored_total,
        params: pass.params,
        playback_blocked: view.playback_blocked,
        exhausted: view.exhausted,
        authority: view.authority,
    };
    let current = compute_snapshot(&post, input);
    let current = with_causal_eta(current, pass.delivery, pass.focus_generation, id);
    let event = event_for(id, pass.emitted.get(id), current.clone());
    pass.emitted.insert(id.to_owned(), current);
    match event {
        Some(event) => out.send(event),
        None => true,
    }
}

fn emit_hls(out: &impl EventOut, pass: &mut Pass<'_>, id: &str) -> bool {
    let current = hls_snapshot(pass.segmented.snapshot(id));
    let event = event_for(id, pass.emitted.get(id), current.clone());
    pass.emitted.insert(id.to_owned(), current);
    event.is_none_or(|event| out.send(event))
}
