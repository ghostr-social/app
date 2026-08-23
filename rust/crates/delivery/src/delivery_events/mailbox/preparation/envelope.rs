use crate::delivery_events::{PlayerPreparationDisposition, PlayerPreparationReport};
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct PlayerPreparationEnvelope {
    report: PlayerPreparationReport,
    completion: Option<oneshot::Sender<PlayerPreparationDisposition>>,
}

impl PlayerPreparationEnvelope {
    pub(super) fn new(
        report: PlayerPreparationReport,
        completion: Option<oneshot::Sender<PlayerPreparationDisposition>>,
    ) -> Self {
        Self { report, completion }
    }

    pub fn report(&self) -> &PlayerPreparationReport {
        &self.report
    }

    pub(super) fn complete(mut self, disposition: PlayerPreparationDisposition) {
        if let Some(completion) = self.completion.take() {
            let _ = completion.send(disposition);
        }
    }
}

impl Drop for PlayerPreparationEnvelope {
    fn drop(&mut self) {
        if let Some(completion) = self.completion.take() {
            let _ = completion.send(PlayerPreparationDisposition::Unavailable);
        }
    }
}
