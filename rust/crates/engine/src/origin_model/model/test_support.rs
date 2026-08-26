use super::*;
use crate::origin_model::ColdStartSelector;

impl OriginModel {
    pub(crate) fn register_cold_start(
        &mut self,
        selector: ColdStartSelector,
        prior: ColdStartPrior,
    ) {
        if self.priors.len() == PRIOR_CAP {
            self.priors.remove(0);
        }
        self.priors.push(PriorRegistration { selector, prior });
    }
}
