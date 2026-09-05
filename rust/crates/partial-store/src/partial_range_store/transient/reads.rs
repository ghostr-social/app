use super::PartialRangeStore;
use crate::partial_range_completion::{Completion, IntegrityMismatch};
use anyhow::{ensure, Result};
use core::ops::Range;
use sha2::Digest as _;

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn read_transient(
        &self,
        key: &str,
        span: &Range<u64>,
    ) -> Option<Option<Vec<u8>>> {
        let mut entries = self.entries.lock().await;
        let responses = self.transient_responses.lock().await;
        let response = responses.get(key)?;
        if let Some(entry) = entries.get_mut(key) {
            entry.touched = self.tick();
        }
        let present = span.start < span.end && span.end <= response.bytes.len() as u64;
        Some(present.then(|| response.bytes[span.start as usize..span.end as usize].to_vec()))
    }

    pub(in crate::partial_range_store) async fn judge_transient(
        &self,
        key: &str,
        advertised: Option<&str>,
    ) -> Option<Result<Completion>> {
        let responses = self.transient_responses.lock().await;
        let response = responses.get(key)?;
        Some(judge(response, advertised))
    }
}

fn judge(response: &super::TransientResponse, advertised: Option<&str>) -> Result<Completion> {
    ensure!(
        response.complete,
        "cannot verify an incomplete transient response"
    );
    let Some(advertised) = advertised else {
        return Ok(Completion::Unverified);
    };
    let actual = format!("{:x}", response.digest.clone().finalize());
    ensure!(actual.eq_ignore_ascii_case(advertised), IntegrityMismatch);
    Ok(Completion::Verified)
}
