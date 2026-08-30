use super::DeliveryState;
use crate::delivery_events::{DeliveryFocus, FocusTransition};
use ghostr_engine::adaptive::CurrentAuthority;
use ghostr_engine::PostId;

#[cfg(test)]
#[path = "provisional_handoff/scope_test.rs"]
mod scope_test;

#[derive(Default)]
pub(super) struct ProvisionalFocusHandoff {
    current: Option<PostId>,
    posts: Vec<PostId>,
    admitted_through_ms: u64,
}

impl DeliveryState {
    pub(super) fn reconcile_provisional_handoff(
        &mut self,
        update: &DeliveryFocus,
        same_current: bool,
        observed_at_ms: u64,
    ) {
        let authority = self.current_authority;
        if !same_current || !carries_handoff(authority, update.transition) {
            self.provisional_focus_handoff.clear();
            return;
        }
        if authority == CurrentAuthority::Provisional {
            self.begin_provisional_handoff(observed_at_ms);
        }
    }

    fn begin_provisional_handoff(&mut self, observed_at_ms: u64) {
        let current = self.focus.current().cloned();
        let future = self.provisional_future_posts();
        self.provisional_focus_handoff
            .begin(current, future, observed_at_ms);
    }

    pub(crate) fn provisional_handoff_rank(
        &self,
        post: &PostId,
        launched_at_ms: u64,
    ) -> Option<usize> {
        self.provisional_focus_handoff
            .rank(self.focus.current(), post, launched_at_ms)
    }
}

fn carries_handoff(authority: CurrentAuthority, transition: FocusTransition) -> bool {
    matches!(
        (authority, transition),
        (
            CurrentAuthority::Provisional,
            FocusTransition::UserNavigation
        ) | (CurrentAuthority::Provisional, FocusTransition::RosterChange)
            | (CurrentAuthority::Canonical, FocusTransition::RosterChange)
    )
}

impl ProvisionalFocusHandoff {
    fn begin(&mut self, current: Option<PostId>, future: Vec<PostId>, observed_at_ms: u64) {
        self.posts = future;
        self.current = current;
        self.admitted_through_ms = observed_at_ms;
        if self.posts.is_empty() {
            self.clear();
        }
    }

    pub(super) fn clear(&mut self) {
        self.current = None;
        self.posts.clear();
        self.admitted_through_ms = 0;
    }

    fn rank(&self, current: Option<&PostId>, post: &PostId, launched_at_ms: u64) -> Option<usize> {
        (self.current.as_ref() == current && launched_at_ms <= self.admitted_through_ms)
            .then(|| self.posts.iter().position(|candidate| candidate == post))
            .flatten()
    }
}
