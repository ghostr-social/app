use super::{
    RecordedActionForecast, RecordedActionValue, RecordedCompletionTimes, RecordedResourceCost,
    RecordedSearchAction,
};
use crate::adaptive::{ActionForecast, ActionNode, ActionValue, CompletionTimes, ResourceCost};
use crate::PostId;

impl RecordedSearchAction {
    pub(super) fn restore(&self) -> Option<ActionNode> {
        let node = ActionNode::new(
            self.planner_action_id,
            PostId::new(&self.post_id),
            self.kind.restore()?,
            self.value.restore(),
        )
        .with_resources(self.resources.restore())
        .with_forecast(self.forecast.restore())
        .with_origin_admission_intent(self.origin_admission_intent.restore())
        .requiring(&self.dependencies);
        self.attach_source(self.attach_authority(node))
    }

    fn attach_authority(&self, node: ActionNode) -> ActionNode {
        match self.authorized_resources {
            Some(value) => node.with_resource_authority(value.restore()),
            None => node,
        }
    }

    fn attach_source(&self, node: ActionNode) -> Option<ActionNode> {
        let Some(source) = self.request_source_id.as_deref() else {
            return (self.resources.requests == 0).then_some(node);
        };
        let node = node.with_origin(source);
        node.request_authority().is_some().then_some(node)
    }
}

impl RecordedActionValue {
    fn restore(self) -> ActionValue {
        ActionValue {
            delay_loss_micros: self.delay_loss_micros,
            reserve_gain_micros: self.reserve_gain_micros,
            information_value_micros: self.information_value_micros,
            exploration_micros: self.exploration_micros,
            cache_gain_micros: self.cache_gain_micros,
            tail_risk_micros: self.tail_risk_micros,
            cvar_micros: self.cvar_micros,
            rank_cost_micros: self.rank_cost_micros,
        }
    }
}

impl RecordedActionForecast {
    fn restore(self) -> ActionForecast {
        ActionForecast::new(
            self.completion.restore(),
            self.success_bps,
            self.ready_playback_ms,
        )
        .with_quality(self.quality_gain_micros)
        .with_cache_reuse(self.cache_reuse_bps)
    }
}

impl RecordedCompletionTimes {
    fn restore(self) -> CompletionTimes {
        CompletionTimes::new(self.expected_ms, self.p95_ms, self.p99_ms, self.cvar_ms)
    }
}

impl RecordedResourceCost {
    pub(super) const fn restore(self) -> ResourceCost {
        ResourceCost::new(
            self.network_bytes,
            self.storage_bytes,
            self.cpu_ms,
            self.requests,
        )
    }
}
