use super::*;

pub(in crate::segmented) use blocks::axiom_test_support::{StageBlock, StoredStage};

impl SegmentedCache {
    pub(crate) fn publish_test_hls(&self, post: &PostId, generation: u64, url: &str, body: &[u8]) {
        let object = crate::segmented::prepare::PreparedObject {
            request_url: url.to_owned(),
            final_url: url::Url::parse(url).expect("valid test HLS URL"),
            body: std::sync::Arc::from(body),
            content_type: Some("application/vnd.apple.mpegurl".to_owned()),
            cache: Default::default(),
        };
        assert!(self.mark_stage_preparing(post, generation, 1, body.len() as u64));
        assert!(self.store_stage_object(post, generation, object).is_some());
        assert!(self.mark_stage_ready(post, generation));
    }
}
