use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::leases::StoreLease;
use ghostr_partial_store::partial_range_store::PartialRangeStore;

#[derive(Default)]
pub(crate) struct FocusedStoreLease {
    lease: Option<StoreLease>,
}

impl FocusedStoreLease {
    pub(crate) fn pin(&mut self, store: &PartialRangeStore, post: Option<&PostId>) {
        self.lease = post.map(|post| store.lease(post.as_str()));
    }
}
