use crate::cache::cacheable_event;
use crate::cache::database::MAX_CACHED_EVENTS;
use crate::cache::session::ViewerScope;
use anyhow::{ensure, Context as _};
use nostr_sdk::{Event, PublicKey};
use serde::{Deserialize, Serialize};

pub(crate) const MAX_SNAPSHOT_BYTES: usize = 32 * 1024 * 1024;
const SNAPSHOT_VERSION: u8 = 1;

#[derive(Deserialize, Serialize)]
struct DiskSnapshot {
    version: u8,
    viewer: DiskViewer,
    events: Vec<Event>,
}

#[derive(Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "scope", content = "public_key")]
enum DiskViewer {
    SignedOut,
    SignedIn(PublicKey),
}

pub(super) fn encode(viewer: ViewerScope, events: &[Event]) -> anyhow::Result<Vec<u8>> {
    let mut snapshot = DiskSnapshot {
        version: SNAPSHOT_VERSION,
        viewer: disk_viewer(viewer)?,
        events: Vec::new(),
    };
    let mut bytes = encoded_size(&snapshot)?;
    for event in events.iter().take(MAX_CACHED_EVENTS) {
        let event_bytes = serde_json::to_vec(event).context("encode cached event")?;
        if fits(bytes, event_bytes.len(), snapshot.events.len()) {
            bytes += event_bytes.len() + usize::from(!snapshot.events.is_empty());
            snapshot.events.push(event.clone());
        }
    }
    let body = serde_json::to_vec(&snapshot).context("encode event cache snapshot")?;
    ensure!(
        body.len() <= MAX_SNAPSHOT_BYTES,
        "event cache snapshot exceeded its byte bound"
    );
    Ok(body)
}

pub(super) fn decode(bytes: &[u8], viewer: ViewerScope) -> anyhow::Result<Vec<Event>> {
    ensure!(
        bytes.len() <= MAX_SNAPSHOT_BYTES,
        "event cache snapshot is oversized"
    );
    let snapshot: DiskSnapshot =
        serde_json::from_slice(bytes).context("decode event cache snapshot")?;
    ensure!(
        snapshot.version == SNAPSHOT_VERSION,
        "unsupported event cache snapshot version"
    );
    ensure!(
        snapshot.viewer == disk_viewer(viewer)?,
        "event cache snapshot belongs to another viewer"
    );
    ensure!(
        snapshot.events.len() <= MAX_CACHED_EVENTS,
        "event cache snapshot has too many events"
    );
    ensure!(
        snapshot.events.iter().all(valid_event),
        "event cache snapshot contains an unverified event"
    );
    Ok(snapshot.events)
}

fn encoded_size(snapshot: &DiskSnapshot) -> anyhow::Result<usize> {
    serde_json::to_vec(snapshot)
        .map(|body| body.len())
        .context("encode empty event cache snapshot")
}

fn fits(current: usize, event: usize, count: usize) -> bool {
    current
        .saturating_add(event)
        .saturating_add(usize::from(count > 0))
        <= MAX_SNAPSHOT_BYTES
}

fn valid_event(event: &Event) -> bool {
    cacheable_event(event) && event.verify().is_ok()
}

fn disk_viewer(viewer: ViewerScope) -> anyhow::Result<DiskViewer> {
    match viewer {
        ViewerScope::SignedOut => Ok(DiskViewer::SignedOut),
        ViewerScope::SignedIn(public_key) => Ok(DiskViewer::SignedIn(public_key)),
        ViewerScope::Unknown => anyhow::bail!("unknown viewer cannot own an event cache snapshot"),
    }
}
