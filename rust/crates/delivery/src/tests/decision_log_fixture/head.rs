use crate::manager::plan::PlannedWork;
use ghostr_engine::adaptive::{
    ActionKind, ActionNode, ActionValue, GeneratedAction, PlannerCommand, ResourceCost,
    RetainedSearchPlan, SearchDecision,
};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

const ACTION_ID: u16 = 91;
const SOURCE: &str = "https://probe.example/video.mp4";

pub(crate) fn work() -> PlannedWork {
    let mut planned = super::work();
    let action = action();
    let warp = planned.warp.as_mut().expect("WARP decision");
    warp.generated.actions = vec![action.clone()];
    warp.selected = Some(action.clone());
    warp.search = search(&action);
    warp.evaluation = None;
    warp.admissible_action_ids = vec![ACTION_ID];
    warp.pruned_action_ids.clear();
    planned
}

pub(crate) fn identity() -> TransferIdentity {
    identity_for("probe", SOURCE)
}

pub(crate) fn wrong_post_identity() -> TransferIdentity {
    identity_for("other", SOURCE)
}

pub(crate) fn wrong_source_identity() -> TransferIdentity {
    identity_for("probe", "https://other.example/video.mp4")
}

fn identity_for(post: &str, source: &str) -> TransferIdentity {
    let post = PostId::new(post);
    let mut catalog = Catalog::new();
    catalog.upsert(
        post.clone(),
        VideoMeta {
            urls: vec![source.into()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: None,
            duration_ms: None,
        },
    );
    catalog.transfer_identity(&post, source).expect("identity")
}

fn action() -> GeneratedAction {
    let post = PostId::new("probe");
    let node = ActionNode::new(
        ACTION_ID,
        post.clone(),
        ActionKind::Head,
        ActionValue::default(),
    )
    .with_resources(ResourceCost::new(0, 0, 0, 1))
    .with_origin(SOURCE);
    GeneratedAction {
        node,
        command: PlannerCommand::ProbeHead {
            post,
            source: SOURCE.into(),
        },
    }
}

fn search(action: &GeneratedAction) -> SearchDecision {
    let plan = RetainedSearchPlan {
        action_ids: vec![ACTION_ID],
        score_micros: 0,
    };
    SearchDecision {
        action: Some(action.node.clone()),
        chosen_plan: Some(plan.clone()),
        committed_actions: 1,
        retained_plans: vec![plan],
        ..SearchDecision::default()
    }
}
