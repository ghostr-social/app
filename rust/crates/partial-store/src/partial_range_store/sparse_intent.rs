use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use anyhow::{ensure, Context as _, Result};
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const VERSION: u8 = 1;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SparseIntent {
    version: u8,
    representation: String,
    source: String,
    generation: SourceGeneration,
    stable_accounted: u64,
    actions: Vec<ActionIntent>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ActionIntent {
    id: u64,
    start: u64,
    end: u64,
}

pub(super) struct SparseIntentAction<'a> {
    pub(super) id: u64,
    pub(super) representation: &'a str,
    pub(super) source: &'a str,
    pub(super) generation: &'a SourceGeneration,
    pub(super) range: ByteRange,
}

pub(super) async fn add(
    paths: &StorePaths,
    key: &str,
    action: SparseIntentAction<'_>,
    stable_accounted: u64,
) -> Result<()> {
    let mut intent = load(paths, key).await?.unwrap_or_else(|| SparseIntent {
        version: VERSION,
        representation: action.representation.to_owned(),
        source: action.source.to_owned(),
        generation: action.generation.clone(),
        stable_accounted,
        actions: Vec::new(),
    });
    ensure_identity(&intent, &action)?;
    if intent.actions.iter().any(|known| known.id == action.id) {
        return Ok(());
    }
    ensure!(
        intent
            .actions
            .iter()
            .all(|known| !overlaps(known, action.range)),
        "sparse action intent overlaps another action"
    );
    intent.actions.push(ActionIntent {
        id: action.id,
        start: action.range.start,
        end: action.range.end,
    });
    intent.actions.sort_by_key(|known| known.start);
    save(paths, key, &intent).await
}

pub(super) async fn commit(
    paths: &StorePaths,
    key: &str,
    action_id: u64,
    stable_accounted: u64,
) -> Result<()> {
    let mut intent = load(paths, key)
        .await?
        .context("sparse write intent is missing")?;
    let before = intent.actions.len();
    intent.actions.retain(|known| known.id != action_id);
    ensure!(
        intent.actions.len() + 1 == before,
        "sparse action intent is missing"
    );
    intent.stable_accounted = stable_accounted;
    if intent.actions.is_empty() {
        remove(paths, key).await
    } else {
        save(paths, key, &intent).await
    }
}

pub(super) async fn exists(paths: &StorePaths, key: &str) -> Result<bool> {
    Ok(load(paths, key).await?.is_some())
}

pub(super) async fn cleanup_bound(paths: &StorePaths, key: &str) -> u64 {
    match load(paths, key).await {
        Ok(Some(intent)) => intent
            .actions
            .iter()
            .fold(intent.stable_accounted, |sum, action| {
                sum.saturating_add(action.end - action.start)
            }),
        _ => disk::file_len(&paths.partial(key))
            .await
            .ok()
            .flatten()
            .unwrap_or_default(),
    }
}

async fn load(paths: &StorePaths, key: &str) -> Result<Option<SparseIntent>> {
    let text = match tokio::fs::read_to_string(paths.sparse_intent(key)).await {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("read sparse write intent"),
    };
    let intent: SparseIntent = serde_json::from_str(&text).context("parse sparse write intent")?;
    validate(&intent)?;
    Ok(Some(intent))
}

async fn save(paths: &StorePaths, key: &str, intent: &SparseIntent) -> Result<()> {
    validate(intent)?;
    let bytes = serde_json::to_vec(intent).context("encode sparse write intent")?;
    disk::save_durable(
        &paths.sparse_intent(key),
        &paths.sparse_intent_staging(key),
        &bytes,
    )
    .await
}

pub(super) async fn remove(paths: &StorePaths, key: &str) -> Result<()> {
    disk::remove_if_present(&paths.sparse_intent_staging(key)).await?;
    disk::remove_durable(&paths.sparse_intent(key)).await
}

fn validate(intent: &SparseIntent) -> Result<()> {
    ensure!(intent.version == VERSION, "unsupported sparse write intent");
    ensure!(
        !intent.representation.is_empty(),
        "missing sparse representation"
    );
    ensure!(!intent.source.is_empty(), "missing sparse source");
    ensure!(!intent.actions.is_empty(), "empty sparse write intent");
    let mut ids = HashSet::new();
    for (index, action) in intent.actions.iter().enumerate() {
        ensure!(ids.insert(action.id), "duplicate sparse action");
        ensure!(action.start < action.end, "empty sparse action range");
        ensure!(
            action.end - action.start <= ghostr_engine::adaptive::REQUEST_SLICE_BYTES,
            "sparse action exceeds its cancellation block"
        );
        ensure!(
            action.end <= intent.generation.total_bytes(),
            "sparse action exceeds its generation"
        );
        if let Some(previous) = index.checked_sub(1).and_then(|i| intent.actions.get(i)) {
            ensure!(previous.end <= action.start, "overlapping sparse actions");
        }
    }
    Ok(())
}

fn ensure_identity(intent: &SparseIntent, action: &SparseIntentAction<'_>) -> Result<()> {
    ensure!(
        intent.representation == action.representation,
        "sparse representation changed"
    );
    ensure!(intent.source == action.source, "sparse source changed");
    ensure!(
        intent.generation == *action.generation,
        "sparse generation changed"
    );
    Ok(())
}

fn overlaps(known: &ActionIntent, range: ByteRange) -> bool {
    known.start < range.end && range.start < known.end
}
