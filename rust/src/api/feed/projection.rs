//! One feed projection shared by mobile FFI and the debug web adapter.

use crate::api::feed::state::FeedState;
use crate::api::feed_types::{FfiFeedPost, FfiFeedStage};
use crate::discovery::feed::store::FeedId;

pub(crate) struct FeedProjection {
    pub stage: FfiFeedStage,
    pub posts: Vec<FfiFeedPost>,
}

pub(crate) fn project(state: &FeedState, feed: FeedId) -> FeedProjection {
    FeedProjection {
        stage: state.stage(feed),
        posts: state.snapshot(feed),
    }
}
