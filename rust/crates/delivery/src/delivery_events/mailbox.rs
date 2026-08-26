use super::{DeliveryCandidate, DeliveryCommand, FocusGenerationGuard};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc;

mod control;
mod preparation;
mod presentation;
mod receiver;
mod sender;
pub(crate) use preparation::PlayerPreparationEnvelope;

#[derive(Clone, Debug)]
pub(super) struct MailboxSender {
    state: Arc<Mutex<MailboxState>>,
    control_wake: mpsc::Sender<()>,
    candidate_wake: mpsc::Sender<()>,
    preparation_wake: mpsc::Sender<()>,
}

pub struct MailboxReceiver {
    state: Arc<Mutex<MailboxState>>,
    control_wake: mpsc::Receiver<()>,
    candidate_wake: mpsc::Receiver<()>,
    preparation_wake: mpsc::Receiver<()>,
}

#[derive(Debug)]
struct MailboxState {
    controls: VecDeque<DeliveryCommand>,
    candidates: VecDeque<DeliveryCandidate>,
    candidate_capacity: usize,
    focus_generations: FocusGenerationGuard,
    next_network_profile_generation: u64,
    preparations: preparation::PreparationMailbox,
    presentations: presentation::PresentationMailbox,
}

pub(super) fn channel(capacity: usize) -> (MailboxSender, MailboxReceiver) {
    let state = Arc::new(Mutex::new(MailboxState {
        controls: VecDeque::new(),
        candidates: VecDeque::new(),
        candidate_capacity: capacity.max(1),
        focus_generations: FocusGenerationGuard::default(),
        next_network_profile_generation: 0,
        preparations: preparation::PreparationMailbox::default(),
        presentations: presentation::PresentationMailbox::default(),
    }));
    let (control_sender, control_wake) = mpsc::channel(1);
    let (candidate_sender, candidate_wake) = mpsc::channel(1);
    let (preparation_sender, preparation_wake) = mpsc::channel(1);
    (
        MailboxSender {
            state: Arc::clone(&state),
            control_wake: control_sender,
            candidate_wake: candidate_sender,
            preparation_wake: preparation_sender,
        },
        MailboxReceiver {
            state,
            control_wake,
            candidate_wake,
            preparation_wake,
        },
    )
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
