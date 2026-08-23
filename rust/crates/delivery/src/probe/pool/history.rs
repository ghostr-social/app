use ghostr_engine::catalog::Catalog;
#[cfg(test)]
use ghostr_engine::representation::HttpGenerationLease;
use ghostr_engine::representation::{HttpGenerationStamp, TransferIdentity};

pub(super) struct CompletedHeadProbe {
    stamp: Option<HttpGenerationStamp>,
    observed_size: bool,
}

impl CompletedHeadProbe {
    #[cfg(test)]
    pub(super) fn for_test(generation: Option<HttpGenerationLease>) -> Self {
        Self {
            stamp: generation.map(HttpGenerationStamp::from_trusted),
            observed_size: true,
        }
    }

    pub(super) const fn new(stamp: HttpGenerationStamp, observed_size: bool) -> Self {
        Self {
            stamp: Some(stamp),
            observed_size,
        }
    }

    pub(super) fn current(&self, catalog: &Catalog, identity: &TransferIdentity) -> bool {
        let Some(entry) = catalog.lookup(identity.post()) else {
            return false;
        };
        if entry
            .binding()
            .transfer(identity.source().as_str())
            .as_ref()
            != Some(identity)
        {
            return false;
        }
        self.stamp == catalog.http_generation_stamp_for(identity)
    }

    pub(super) const fn observed_size(&self) -> bool {
        self.observed_size
    }
}
