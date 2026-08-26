use super::*;

use crate::scheduler::hunt::HuntToken;

impl DiscoveryHandle {
    pub(crate) fn focus(&self, context: FeedContext) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Feed(FeedCommand::Focus(context)));
    }

    pub(crate) fn background(&self, context: FeedContext, request: DiscoveryRequest) {
        let _ = self
            .sender
            .send(DiscoveryCommand::Work(WorkCommand::Background {
                context,
                request,
            }));
    }

    pub(crate) fn inject_retry(&self, context: FeedContext, token: u64) {
        let _ = self.sender.send(DiscoveryCommand::Work(WorkCommand::Retry {
            context,
            token: HuntToken(token),
        }));
    }
}
