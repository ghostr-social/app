//! Taking stock of the store root at startup. The directory outlives
//! the process, so a fresh run must adopt what the last one left there:
//! every completed and partial file with a usable manifest is accounted
//! for and reusable before anything asks for it. Without this the store
//! believes it holds nothing — it re-downloads bytes that are already
//! on disk, and cannot evict files it does not know it owns.

use crate::video::partial_range_disk::{self as disk, Entry};
use crate::video::partial_range_paths::{validate_key, StorePaths};
use crate::video::partial_range_store::PartialRangeStore;
use anyhow::Result;
use log::warn;
use std::collections::BTreeSet;
use std::path::Path;

impl PartialRangeStore {
    /// Adopts the store root as an earlier run left it. Partial data
    /// nothing can be resumed from — bytes whose manifest is missing,
    /// unreadable or shorter than the file — is given back instead of
    /// leaking, since the space it holds would never be reclaimed.
    pub async fn load_existing(&self) -> Result<()> {
        let mut entries = self.entries.lock().await;
        for key in stored_keys(&self.root).await {
            if entries.contains_key(&key) {
                continue;
            }
            match reusable(&self.paths, &key).await {
                Ok(Some(entry)) => self.adopt(&mut entries, key, entry).await,
                Ok(None) => self.drop_unusable(&mut entries, &key).await,
                Err(error) => warn!("Video store could not reload {key}: {error:#}"),
            }
        }
        Ok(())
    }

    async fn adopt(&self, entries: &mut super::Entries, key: String, entry: Entry) {
        self.credit(entry.accounted).await;
        entries.insert(key, entry);
    }

    async fn drop_unusable(&self, entries: &mut super::Entries, key: &str) {
        if let Err(error) = self.discard(entries, key).await {
            warn!("Video store could not clear unusable {key}: {error:#}");
        }
    }
}

/// What the store may keep: a completed file, or a partial file its
/// manifest actually describes. Ranges are only ever committed after
/// their bytes are flushed, so a file shorter than the manifest claims
/// is a torn write and nothing may be resumed from it.
async fn reusable(paths: &StorePaths, key: &str) -> Result<Option<Entry>> {
    let entry = disk::load_entry(paths, key).await?;
    if entry.completion.is_some() {
        return Ok(Some(entry));
    }
    let claimed = entry.manifest.ranges().last().map_or(0, |span| span.end);
    let stored = disk::file_len(&paths.partial(key)).await?.unwrap_or(0);
    Ok((claimed > 0 && stored >= claimed).then_some(entry))
}

/// Every key the root holds. File names are `{key}.{extension}` and a
/// key may carry no dot, so the key is the leading segment.
async fn stored_keys(root: &Path) -> BTreeSet<String> {
    let Ok(mut listing) = tokio::fs::read_dir(root).await else {
        return BTreeSet::new();
    };
    let mut keys = BTreeSet::new();
    while let Ok(Some(item)) = listing.next_entry().await {
        let name = item.file_name();
        let key = name.to_str().and_then(|name| name.split('.').next());
        if let Some(key) = key.filter(|key| validate_key(key).is_ok()) {
            keys.insert(key.to_owned());
        }
    }
    keys
}
