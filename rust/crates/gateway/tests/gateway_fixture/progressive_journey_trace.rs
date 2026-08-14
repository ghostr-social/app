use ghostr_delivery::delivery_events::{DeliveryFocus, DeliveryPlayback};
use ghostr_engine::playback::PlaybackPhase;
use ghostr_engine::PostId;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusTrace {
    pub posts: Vec<PostId>,
    pub current_index: usize,
}

#[derive(Clone, Default)]
pub struct ProgressiveJourneyTrace {
    state: Arc<Mutex<TraceState>>,
}

#[derive(Default)]
struct TraceState {
    focuses: Vec<FocusTrace>,
    observations: Vec<DeliveryPlayback>,
    first_frames: Vec<PostId>,
    cancellations: Vec<PostId>,
}

impl ProgressiveJourneyTrace {
    pub fn record_focus(&self, focus: &DeliveryFocus) {
        self.state
            .lock()
            .expect("journey trace")
            .focuses
            .push(FocusTrace {
                posts: focus.items.iter().map(|item| item.post.clone()).collect(),
                current_index: focus.current_index,
            });
    }

    pub fn record_observation(&self, playback: DeliveryPlayback) {
        self.state
            .lock()
            .expect("journey trace")
            .observations
            .push(playback);
    }

    pub fn record_first_frame(&self, post: PostId) {
        self.state
            .lock()
            .expect("journey trace")
            .first_frames
            .push(post);
    }

    pub fn record_cancellation(&self, post: PostId) {
        self.state
            .lock()
            .expect("journey trace")
            .cancellations
            .push(post);
    }

    pub fn focuses(&self) -> Vec<FocusTrace> {
        self.state.lock().expect("journey trace").focuses.clone()
    }

    pub fn observations(&self) -> Vec<DeliveryPlayback> {
        self.state
            .lock()
            .expect("journey trace")
            .observations
            .clone()
    }

    pub fn first_frames(&self) -> Vec<PostId> {
        self.state
            .lock()
            .expect("journey trace")
            .first_frames
            .clone()
    }

    pub fn cancellations(&self) -> Vec<PostId> {
        self.state
            .lock()
            .expect("journey trace")
            .cancellations
            .clone()
    }

    pub fn stalls(&self) -> Vec<PostId> {
        self.observations()
            .into_iter()
            .filter(|playback| playback.observation.phase() == PlaybackPhase::NetworkStalled)
            .map(|playback| playback.session.post().clone())
            .collect()
    }
}
