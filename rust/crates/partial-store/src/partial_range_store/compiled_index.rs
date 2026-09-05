//! Small derived records use the same checksummed storage and capacity ledger
//! as payload, with independent keys and an additional bounded index envelope.
use super::{Entries, PartialRangeStore};
use anyhow::{anyhow, Result};
use ghostr_engine::media_timeline::{compiled, MediaTimeline};

mod key;
pub use key::CompiledIndexKey;

const PREFIX: &str = "warp-index-";
const MAX_RECORDS: usize = 128;
const MAX_BYTES: u64 = 16 * 1024 * 1024;

impl PartialRangeStore {
    /// Call only after current access and response retention policy allow reuse.
    ///
    /// # Errors
    /// Returns storage errors or rejects an invalid/oversized compiled record.
    pub async fn retain_compiled_index(
        &self,
        key: &CompiledIndexKey,
        timeline: &MediaTimeline,
    ) -> Result<()> {
        let bytes = compiled::encode(timeline).map_err(|error| anyhow!("index: {error:?}"))?;
        compiled::decode(&bytes, key.total).map_err(|error| anyhow!("index: {error:?}"))?;
        let _update = self.update_key(&key.storage).await?;
        let mut entries = self.entries.lock().await;
        self.discard(&mut entries, &key.storage).await?;
        self.make_index_room(&mut entries, bytes.len() as u64)
            .await?;
        self.require_headroom(bytes.len() as u64).await?;
        self.write_range_locked(&mut entries, &key.storage, 0, &bytes)
            .await
    }

    /// Reads structural facts after independently validating this source binding.
    /// This never asserts that any source media bytes are present or playable.
    ///
    /// # Errors
    /// Returns errors from the underlying checksummed store.
    pub async fn compiled_index(&self, key: &CompiledIndexKey) -> Result<Option<MediaTimeline>> {
        let spans = self.present_ranges(&key.storage).await?;
        let [span] = spans.as_slice() else {
            return Ok(None);
        };
        if span.start != 0 || span.end > compiled::MAX_ENCODED_BYTES as u64 {
            return Ok(None);
        }
        let Some(bytes) = self.read_range(&key.storage, span.clone()).await? else {
            return Ok(None);
        };
        Ok(compiled::decode(&bytes, key.total).ok())
    }

    async fn make_index_room(&self, entries: &mut Entries, wanted: u64) -> Result<()> {
        let mut records: Vec<_> = entries
            .iter()
            .filter(|(key, entry)| key.starts_with(PREFIX) && entry.accounted > 0)
            .map(|(key, entry)| (entry.touched, key.clone(), entry.accounted))
            .collect();
        records.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
        let mut count = records.len();
        let mut bytes = records.iter().map(|record| record.2).sum::<u64>();
        for (_, key, size) in records {
            if count < MAX_RECORDS && bytes.saturating_add(wanted) <= MAX_BYTES {
                break;
            }
            self.discard(entries, &key).await?;
            count -= 1;
            bytes = bytes.saturating_sub(size);
        }
        Ok(())
    }
}
