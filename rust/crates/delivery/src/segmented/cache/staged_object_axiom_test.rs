use super::*;

impl StagedObject {
    pub(in super::super) fn assembled_with(
        &self,
        block: &PreparedObject,
        offset: u64,
    ) -> Option<PreparedObject> {
        let bytes = usize::try_from(self.continuation_len(block, offset)?).ok()?;
        let mut body = Vec::with_capacity(bytes);
        for known in &self.blocks {
            body.extend_from_slice(known);
        }
        body.extend_from_slice(&block.body);
        if body.len() != bytes {
            return None;
        }
        let cache = self.cache.combined_with(&block.cache);
        Some(self.prepared(body.into(), cache))
    }
}
