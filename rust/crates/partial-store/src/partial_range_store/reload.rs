//! Taking stock of the store root at startup. The directory outlives
//! the process, so a fresh run must adopt what the last one left there:
//! every completed and partial file with a usable manifest is accounted
//! for and reusable before anything asks for it. Without this the store
//! otherwise it re-downloads bytes and cannot evict files it does not know.

use crate::partial_range_disk::{self as disk, Entry};
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::cleanup_debt::CleanupScope;
use crate::partial_range_store::policy_eviction::recovery as policy_recovery;
use crate::partial_range_store::sparse_intent;
use crate::partial_range_store::PartialRangeStore;
use anyhow::Result;
use log::warn;

mod keys;
mod response;
pub(super) use keys::stored_keys;
use response::recover;

pub(super) async fn remove_transaction_files(paths: &StorePaths, key: &str) -> Result<()> {
    response::remove_transaction_files(paths, key).await
}

struct RecoveredEntry {
    entry: Option<Entry>,
    policy_debt: Option<u64>,
    replacement_debt: Option<u64>,
    staging_debt: Option<u64>,
}

impl PartialRangeStore {
    /// Adopts the store root as an earlier run left it. Partial data
    /// nothing can be resumed from — bytes whose manifest is missing,
    /// unreadable or shorter than the file — is given back instead of
    /// leaking, since the space it holds would never be reclaimed.
    pub async fn load_existing(&self) -> Result<()> {
        let mut entries = self.entries.lock().await;
        for key in stored_keys(&self.root).await? {
            if entries.contains_key(&key) {
                continue;
            }
            match reusable(&self.paths, &key).await {
                Ok(RecoveredEntry {
                    entry: Some(entry),
                    policy_debt,
                    replacement_debt,
                    staging_debt,
                }) => {
                    self.adopt(&mut entries, key.clone(), entry).await;
                    self.adopt_cleanup_debt(&key, CleanupScope::PolicyTransaction, policy_debt)
                        .await;
                    self.adopt_cleanup_debt(
                        &key,
                        CleanupScope::ReplacedCanonical,
                        replacement_debt,
                    )
                    .await;
                    self.adopt_cleanup_debt(&key, CleanupScope::StagingOnly, staging_debt)
                        .await;
                }
                Ok(_) => self.drop_unusable(&mut entries, &key).await,
                Err(error) => {
                    warn!("Video store could not reload {key}: {error:#}");
                    self.drop_unusable(&mut entries, &key).await;
                }
            }
        }
        Ok(())
    }

    async fn adopt(&self, entries: &mut super::Entries, key: String, entry: Entry) {
        self.credit(entry.accounted).await;
        entries.insert(key, entry);
    }

    async fn adopt_cleanup_debt(&self, key: &str, scope: CleanupScope, bytes: Option<u64>) {
        let Some(bytes) = bytes else { return };
        if let Err(error) = self.record_cleanup_debt(key, scope, None, bytes).await {
            warn!("Video store could not account cleanup debt for {key}: {error:#}");
        }
    }

    async fn drop_unusable(&self, entries: &mut super::Entries, key: &str) {
        let debt = cleanup_bytes(&self.paths, key).await;
        entries.entry(key.to_owned()).or_insert_with(empty_entry);
        if let Err(error) = self.discard(entries, key).await {
            if let Err(debt_error) = self
                .record_cleanup_debt(key, CleanupScope::CanonicalDirty, None, debt)
                .await
            {
                warn!("Video store could not account unusable {key}: {debt_error:#}");
            }
            warn!("Video store could not clear unusable {key}: {error:#}");
        }
    }
}

fn empty_entry() -> Entry {
    Entry {
        manifest: crate::partial_range_manifest::RangeManifest::default(),
        accounted: 0,
        completion: None,
        touched: 0,
    }
}

async fn cleanup_bytes(paths: &StorePaths, key: &str) -> u64 {
    let mut bytes = sparse_intent::cleanup_bound(paths, key).await;
    bytes = bytes.saturating_add(policy_recovery::cleanup_payload_bytes(paths, key).await);
    for path in [
        paths.partial_staging(key),
        paths.single_response(key),
        paths.completed(key),
    ] {
        bytes = bytes.saturating_add(disk::file_len(&path).await.ok().flatten().unwrap_or(0));
    }
    bytes
}

/// What the store may keep: a completed file, or a partial file its
/// manifest actually describes. Ranges are only ever committed after
/// their bytes are flushed, so a file shorter than the manifest claims
/// is a torn write and nothing may be resumed from it.
async fn reusable(paths: &StorePaths, key: &str) -> Result<RecoveredEntry> {
    let policy = policy_recovery::recover(paths, key).await?;
    if !policy.usable {
        return Ok(RecoveredEntry {
            entry: None,
            policy_debt: policy.cleanup_debt,
            replacement_debt: None,
            staging_debt: None,
        });
    }
    let policy_debt = policy.cleanup_debt;
    let response = recover(paths, key).await?;
    if !response.published && sparse_intent::exists(paths, key).await? {
        return Ok(RecoveredEntry {
            entry: None,
            policy_debt,
            replacement_debt: response.replacement_debt,
            staging_debt: response.staging_debt,
        });
    }
    let entry = disk::load_entry(paths, key).await?;
    if entry.completion.is_some() {
        return Ok(RecoveredEntry {
            entry: Some(entry),
            policy_debt,
            replacement_debt: response.replacement_debt,
            staging_debt: response.staging_debt,
        });
    }
    let claimed = entry.manifest.ranges().last().map_or(0, |span| span.end);
    let stored = disk::file_len(&paths.partial(key)).await?.unwrap_or(0);
    Ok(RecoveredEntry {
        entry: (claimed > 0 && stored >= claimed).then_some(entry),
        policy_debt,
        replacement_debt: response.replacement_debt,
        staging_debt: response.staging_debt,
    })
}
