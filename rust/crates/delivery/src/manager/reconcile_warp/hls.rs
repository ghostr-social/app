use super::WarpDirective;
use crate::delivery_events::DecisionToken;
use crate::manager::selected_commit::{CommitResult, SelectedCommit};
use crate::manager::{time, DeliveryWorker};
use crate::segmented::scheduler::{SegmentedLaunch, SegmentedResourceCommitment};
use ghostr_engine::adaptive::{DecisionOutcome, ResourceCost};

impl DeliveryWorker {
    pub(super) fn launch_selected_hls(
        &mut self,
        directive: &WarpDirective,
        decision: Option<DecisionToken>,
        commit: &mut Option<SelectedCommit>,
    ) {
        let Some(mut launch) = self.hls_launch(directive) else {
            return self.fail_hls_token(decision, "warp_hls_directive_invalid");
        };
        if !self.segmented.can_start(&launch) {
            return self.fail_hls_token(decision, "warp_hls_stage_stale");
        }
        let Some(token) = decision else {
            return;
        };
        let Some((resources, authorized)) = selected_resources(commit.as_ref()) else {
            return self.fail_hls_token(Some(token), "warp_hls_resources_missing");
        };
        let observed_at_ms = time::unix_time_ms();
        if !self
            .commands
            .bind_decision(&token, launch.action, observed_at_ms)
        {
            self.commands
                .resolve_decision_token(&token, failed("decision_binding_rejected"));
            return;
        }
        if self.commit_selected(commit, authorized, observed_at_ms) != CommitResult::Committed {
            return self.fail_bound_hls(launch.action, "warp_resource_commit_rejected");
        }
        launch.resources = resources;
        launch.requests = self.ctx.requests.clone();
        launch.events = self.ctx.events.clone();
        let action = launch.action;
        if !self.segmented.start(launch) {
            self.warp_planner.reconcile_network_reservation(
                resources.reserved_network_bytes(),
                0,
                time::unix_time_ms(),
            );
            return self.fail_bound_hls(action, "warp_hls_launch_rejected");
        }
        self.request_immediate_replan();
    }

    fn hls_launch(&mut self, directive: &WarpDirective) -> Option<SegmentedLaunch> {
        let WarpDirective::HlsBootstrap {
            post,
            stage,
            source,
            cursor,
            maximum_bytes,
            committed_until_ms,
        } = directive
        else {
            return None;
        };
        Some(SegmentedLaunch {
            post: post.clone(),
            stage: *stage,
            source: source.clone(),
            cursor: *cursor,
            maximum_bytes: *maximum_bytes,
            committed_until_ms: *committed_until_ms,
            action: self.downloads.next_action_id(),
            requests: self.ctx.requests.clone(),
            events: self.ctx.events.clone(),
            network_status: self.ctx.network_status.clone(),
            traffic: self.ctx.traffic.clone(),
            resources: SegmentedResourceCommitment::default(),
        })
    }

    fn fail_hls_token(&self, token: Option<DecisionToken>, class: &'static str) {
        if let Some(token) = token {
            self.commands.resolve_decision_token(&token, failed(class));
        }
    }

    fn fail_bound_hls(&self, action: ghostr_engine::ActionId, class: &'static str) {
        self.commands
            .resolve_decision(action, failed(class), time::unix_time_ms());
    }
}

fn selected_resources(
    selected: Option<&SelectedCommit>,
) -> Option<(SegmentedResourceCommitment, ResourceCost)> {
    let (expected, authorized) = selected?.resources()?;
    let resources =
        SegmentedResourceCommitment::new(expected.network_bytes, authorized.network_bytes)?;
    Some((resources, authorized))
}

fn failed(class: &str) -> DecisionOutcome {
    DecisionOutcome::Failed {
        class: class.to_owned(),
        elapsed_ms: 0,
    }
}
