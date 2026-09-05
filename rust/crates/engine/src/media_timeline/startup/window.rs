use super::{closure, media_headers, normalize, ByteRange, MediaTimeline};

impl MediaTimeline {
    /// Exact dependencies for a contiguous interval on every selected track.
    /// Unsupported or excessive closures return no useful-coverage assertion.
    pub fn continuation_dependencies(&self, start_ms: u64, end_ms: u64) -> Option<Vec<ByteRange>> {
        self.startup.as_ref()?;
        if start_ms >= end_ms || end_ms - start_ms > 120_000 || self.media.len() > 8_192 {
            return None;
        }
        let media = closure::between(&self.media, start_ms, end_ms)?;
        let headers = media_headers(&media, &self.media_data)?;
        let mut ranges = self.file_types.clone();
        ranges.push(self.movie?);
        ranges.extend(headers);
        ranges.extend(media);
        Some(normalize(ranges))
    }

    pub fn selected_end_ms(&self) -> Option<u64> {
        self.media
            .chunk_by(|left, right| left.track == right.track)
            .map(|track| {
                track
                    .iter()
                    .map(|sample| sample.time.end_floor_ms())
                    .max()
                    .unwrap_or(0)
            })
            .min()
    }
}
