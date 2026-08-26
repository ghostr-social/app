use super::*;
use ghostr_engine::PostId;

impl PlanEvidenceHistory {
    fn publish(&self, observed_at_ms: u64, plan: AllocationPlan) {
        let context = PlanPublicationContext::new(observed_at_ms, None);
        self.publish_focused(context, plan, Vec::new());
    }
}

impl CommandReceiver {
    pub fn publish_plan(&mut self, observed_at_ms: u64, plan: AllocationPlan) {
        self.plans.publish(observed_at_ms, plan);
    }

    pub fn publish_focused_plan(
        &mut self,
        observed_at_ms: u64,
        current: Option<PostId>,
        plan: AllocationPlan,
    ) {
        self.plans.publish_focused(
            PlanPublicationContext::new(observed_at_ms, current),
            plan,
            Vec::new(),
        );
    }

    pub fn publish_focused_plan_with_startup(
        &mut self,
        observed_at_ms: u64,
        current: Option<PostId>,
        plan: AllocationPlan,
        startup: Option<StartupCertificate>,
    ) {
        self.publish_focused_plan_with_startups(
            observed_at_ms,
            current,
            plan,
            startup.into_iter().collect(),
        );
    }

    pub fn publish_focused_plan_with_startups(
        &mut self,
        observed_at_ms: u64,
        current: Option<PostId>,
        plan: AllocationPlan,
        startups: Vec<StartupCertificate>,
    ) {
        self.plans.publish_focused(
            PlanPublicationContext::new(observed_at_ms, current),
            plan,
            startups,
        );
    }
}
