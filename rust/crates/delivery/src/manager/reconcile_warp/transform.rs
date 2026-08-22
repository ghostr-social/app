use super::WarpDirective;
use crate::delivery_events::DecisionToken;
use crate::manager::selected_commit::{CommitResult, SelectedCommit};
use crate::manager::transforms::TransformRequest;
use crate::manager::{time, DeliveryWorker};
use crate::transform::TransformProfile;
use ghostr_engine::adaptive::{DecisionOutcome, ResourceCost};

struct PreparedTransform {
    request: TransformRequest,
    profile: TransformProfile,
}

impl DeliveryWorker {
    pub(super) async fn transform_selected(
        &mut self,
        directive: &WarpDirective,
        decision: Option<DecisionToken>,
        commit: &mut Option<SelectedCommit>,
    ) {
        let prepared = match self.prepare_transform(directive).await {
            Ok(prepared) => prepared,
            Err(class) => return self.fail_transform_token(decision, class),
        };
        let action = prepared.request.action;
        let bound = match self.bind_transform_decision(decision, action) {
            Ok(bound) => bound,
            Err(()) => return,
        };
        if !self.commit_transform(commit, prepared.profile) {
            return self.fail_bound_transform(action, bound, "warp_resource_commit_rejected");
        }
        let post = prepared.request.binding.post().clone();
        if !self.state.begin_transform(post.clone())
            || !self
                .transforms
                .launch(self.ctx.store.clone(), prepared.request)
        {
            self.state.finish_transform(&post);
            return self.fail_bound_transform(action, bound, "warp_transform_launch_rejected");
        }
        self.request_immediate_replan();
    }

    async fn prepare_transform(
        &mut self,
        directive: &WarpDirective,
    ) -> Result<PreparedTransform, &'static str> {
        let WarpDirective::Transform { post, kind } = directive else {
            return Err("warp_transform_directive_invalid");
        };
        let profile = self
            .transforms
            .profile()
            .ok_or("warp_transform_backend_unavailable")?;
        if profile.kind() != *kind || self.transforms.contains(post) {
            return Err("warp_transform_backend_unavailable");
        }
        let binding = self
            .state
            .catalog()
            .binding(post)
            .ok_or("warp_transform_identity_missing")?;
        let snapshot = self
            .ctx
            .store
            .media_snapshot(post.as_str())
            .await
            .map_err(|_| "warp_transform_input_read_failed")?;
        let total = transform_total(&snapshot, &binding, profile)?;
        Ok(PreparedTransform {
            request: TransformRequest {
                action: self.downloads.next_action_id(),
                binding,
                revision: snapshot.revision(),
                total,
                kind: *kind,
            },
            profile,
        })
    }

    fn bind_transform_decision(
        &self,
        decision: Option<DecisionToken>,
        action: ghostr_engine::ActionId,
    ) -> Result<bool, ()> {
        let Some(token) = decision else {
            return Ok(false);
        };
        if self
            .commands
            .bind_decision(&token, action, time::unix_time_ms())
        {
            return Ok(true);
        }
        self.commands
            .resolve_decision_token(&token, failed("decision_binding_rejected"));
        Err(())
    }

    fn commit_transform(
        &mut self,
        commit: &mut Option<SelectedCommit>,
        profile: TransformProfile,
    ) -> bool {
        let limits = profile.limits();
        let resources = ResourceCost::new(0, limits.output_bytes(), limits.cpu_ms(), 0);
        self.commit_selected(commit, resources, time::unix_time_ms()) == CommitResult::Committed
    }

    fn fail_transform_token(&self, decision: Option<DecisionToken>, class: &'static str) {
        if let Some(token) = decision {
            self.commands.resolve_decision_token(&token, failed(class));
        }
    }

    fn fail_bound_transform(
        &self,
        action: ghostr_engine::ActionId,
        bound: bool,
        class: &'static str,
    ) {
        if bound {
            self.commands
                .resolve_decision(action, failed(class), time::unix_time_ms());
        }
    }
}

fn transform_total(
    snapshot: &ghostr_partial_store::partial_range_store::StoredMediaSnapshot,
    binding: &ghostr_engine::representation::RepresentationBinding,
    profile: TransformProfile,
) -> Result<u64, &'static str> {
    if snapshot.binding() != Some(binding) || !snapshot.is_finalized() {
        return Err("warp_transform_input_not_finalized");
    }
    snapshot
        .total_len()
        .filter(|total| *total > 0 && *total <= profile.limits().input_bytes())
        .ok_or("warp_transform_input_envelope_rejected")
}

fn failed(class: &str) -> DecisionOutcome {
    DecisionOutcome::Failed {
        class: class.to_owned(),
        elapsed_ms: 0,
    }
}
