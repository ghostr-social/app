use super::{
    CandidateAdmission, DeliveryCandidate, DeliveryCommand, DeliveryFocus, FocusAdmission,
    FocusGenerationGuard,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc;

mod control;

#[derive(Clone, Debug)]
pub(super) struct MailboxSender {
    state: Arc<Mutex<MailboxState>>,
    control_wake: mpsc::Sender<()>,
    candidate_wake: mpsc::Sender<()>,
}

pub struct MailboxReceiver {
    state: Arc<Mutex<MailboxState>>,
    control_wake: mpsc::Receiver<()>,
    candidate_wake: mpsc::Receiver<()>,
}

#[derive(Debug)]
struct MailboxState {
    controls: VecDeque<DeliveryCommand>,
    candidates: VecDeque<DeliveryCandidate>,
    candidate_capacity: usize,
    focus_generations: FocusGenerationGuard,
}

pub(super) fn channel(capacity: usize) -> (MailboxSender, MailboxReceiver) {
    let state = Arc::new(Mutex::new(MailboxState {
        controls: VecDeque::new(),
        candidates: VecDeque::new(),
        candidate_capacity: capacity.max(1),
        focus_generations: FocusGenerationGuard::default(),
    }));
    let (control_sender, control_wake) = mpsc::channel(1);
    let (candidate_sender, candidate_wake) = mpsc::channel(1);
    (
        MailboxSender {
            state: Arc::clone(&state),
            control_wake: control_sender,
            candidate_wake: candidate_sender,
        },
        MailboxReceiver {
            state,
            control_wake,
            candidate_wake,
        },
    )
}

impl MailboxSender {
    pub(super) fn send_control(&self, command: DeliveryCommand) -> bool {
        if self.control_wake.is_closed() {
            return false;
        }
        control::replace(&mut self.lock().controls, command);
        signal(&self.control_wake)
    }

    pub(super) fn send_focus(&self, focus: DeliveryFocus) -> FocusAdmission {
        if self.control_wake.is_closed() {
            return FocusAdmission::Closed;
        }
        let mut state = self.lock();
        if !state.focus_generations.accept(focus.generation) {
            return FocusAdmission::Stale;
        }
        control::replace(&mut state.controls, DeliveryCommand::Focus(focus));
        drop(state);
        match signal(&self.control_wake) {
            true => FocusAdmission::Accepted,
            false => FocusAdmission::Closed,
        }
    }

    pub(super) fn send_candidate(&self, candidate: DeliveryCandidate) -> CandidateAdmission {
        if self.candidate_wake.is_closed() {
            return CandidateAdmission::Closed;
        }
        let mut state = self.lock();
        if state.candidates.len() >= state.candidate_capacity {
            return CandidateAdmission::Saturated;
        }
        state.candidates.push_back(candidate);
        drop(state);
        match signal(&self.candidate_wake) {
            true => CandidateAdmission::Accepted,
            false => CandidateAdmission::Closed,
        }
    }

    fn lock(&self) -> MutexGuard<'_, MailboxState> {
        lock(&self.state)
    }
}

impl MailboxReceiver {
    pub(crate) async fn changed(&mut self) -> bool {
        if self.has_ready() {
            return true;
        }
        let control_open = !self.control_wake.is_closed();
        let candidate_open = !self.candidate_wake.is_closed();
        tokio::select! {
            value = self.control_wake.recv(), if control_open => value.is_some(),
            value = self.candidate_wake.recv(), if candidate_open => value.is_some(),
            else => false,
        }
    }

    pub(super) fn try_control(&mut self) -> Option<DeliveryCommand> {
        self.lock().controls.pop_front()
    }

    pub(super) fn try_candidate(&mut self) -> Option<DeliveryCandidate> {
        self.lock().candidates.pop_front()
    }

    pub(super) fn has_control(&self) -> bool {
        !self.lock().controls.is_empty()
    }

    pub(super) fn has_candidate(&self) -> bool {
        !self.lock().candidates.is_empty()
    }

    pub(super) fn clear(&mut self) {
        let mut state = self.lock();
        state.controls.clear();
        state.candidates.clear();
    }

    fn has_ready(&self) -> bool {
        self.has_control() || self.has_candidate()
    }

    fn lock(&self) -> MutexGuard<'_, MailboxState> {
        lock(&self.state)
    }
}

fn signal(sender: &mpsc::Sender<()>) -> bool {
    match sender.try_send(()) {
        Ok(()) | Err(mpsc::error::TrySendError::Full(())) => true,
        Err(mpsc::error::TrySendError::Closed(())) => false,
    }
}

fn lock(state: &Arc<Mutex<MailboxState>>) -> MutexGuard<'_, MailboxState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
