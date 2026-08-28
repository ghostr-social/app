use super::ActionValue;
use crate::adaptive::{ResourceCost, ResourcePrices};

impl ActionValue {
    pub const fn from_net_micros(value: i64) -> Self {
        Self {
            delay_loss_micros: value,
            reserve_gain_micros: 0,
            information_value_micros: 0,
            exploration_micros: 0,
            cache_gain_micros: 0,
            tail_risk_micros: 0,
            cvar_micros: 0,
            rank_cost_micros: 0,
        }
    }

    pub(crate) const fn for_origin_intent(
        mut self,
        intent: crate::origin_model::OriginAdmissionIntent,
    ) -> Self {
        if matches!(intent, crate::origin_model::OriginAdmissionIntent::Delivery) {
            self.exploration_micros = 0;
        }
        self
    }

    pub(crate) fn total(self, resources: ResourceCost, prices: ResourcePrices) -> i64 {
        let benefits = self
            .delay_loss_micros
            .saturating_add(self.reserve_gain_micros)
            .saturating_add(self.information_value_micros)
            .saturating_add(self.exploration_micros)
            .saturating_add(self.cache_gain_micros);
        benefits
            .saturating_sub(prices.cost(resources))
            .saturating_sub(self.tail_risk_micros)
            .saturating_sub(self.cvar_micros)
            .saturating_sub(self.rank_cost_micros)
    }
}
