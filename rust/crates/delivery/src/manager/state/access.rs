use super::DeliveryState;
use ghostr_engine::adaptive::NavigationSnapshot;
use ghostr_engine::catalog::Catalog;
use ghostr_engine::focus::FocusState;
use ghostr_engine::EngineParams;

impl DeliveryState {
    pub(crate) fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub(crate) fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }

    pub(crate) fn focus(&self) -> &FocusState {
        &self.focus
    }

    pub(in crate::manager) fn params(&self) -> &EngineParams {
        &self.effective
    }

    pub(crate) fn concurrency(&self) -> usize {
        self.effective.concurrency(self.level)
    }

    pub(crate) fn navigation(&self, observed_at_ms: u64) -> NavigationSnapshot {
        self.navigation.snapshot(observed_at_ms)
    }
}
