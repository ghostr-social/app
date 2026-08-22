use super::{signal, MailboxReceiver, MailboxSender};
use crate::delivery_events::{PlayerPreparationIngress, PlayerPreparationReport};
use ghostr_engine::PostId;
use std::collections::{HashMap, VecDeque};

const PLAYER_PREPARATION_CAPACITY: usize = 4;

#[derive(Debug, Default)]
pub(super) struct PreparationMailbox {
    order: VecDeque<PostId>,
    reports: HashMap<PostId, PlayerPreparationReport>,
}

impl PreparationMailbox {
    fn insert(&mut self, report: PlayerPreparationReport) -> PlayerPreparationIngress {
        let post = report.post().clone();
        if let Some(previous) = self.reports.get(&post) {
            if !report.supersedes(previous) {
                return PlayerPreparationIngress::Stale;
            }
            self.reports.insert(post, report);
            return PlayerPreparationIngress::Accepted;
        }
        if self.reports.len() >= PLAYER_PREPARATION_CAPACITY {
            return PlayerPreparationIngress::Saturated;
        }
        self.order.push_back(post.clone());
        self.reports.insert(post, report);
        PlayerPreparationIngress::Accepted
    }

    fn pop(&mut self) -> Option<PlayerPreparationReport> {
        let post = self.order.pop_front()?;
        self.reports.remove(&post)
    }

    pub(super) fn clear(&mut self) {
        self.order.clear();
        self.reports.clear();
    }

    fn is_empty(&self) -> bool {
        self.reports.is_empty()
    }
}

impl MailboxSender {
    pub(crate) fn send_player_preparation(
        &self,
        report: PlayerPreparationReport,
    ) -> PlayerPreparationIngress {
        if self.preparation_wake.is_closed() {
            return PlayerPreparationIngress::Closed;
        }
        let admission = self.lock().preparations.insert(report);
        if admission != PlayerPreparationIngress::Accepted {
            return admission;
        }
        match signal(&self.preparation_wake) {
            true => admission,
            false => PlayerPreparationIngress::Closed,
        }
    }
}

impl MailboxReceiver {
    pub(crate) fn try_player_preparation(&mut self) -> Option<PlayerPreparationReport> {
        self.lock().preparations.pop()
    }

    pub(crate) fn has_player_preparation(&self) -> bool {
        !self.lock().preparations.is_empty()
    }
}
