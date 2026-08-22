use super::DeliveryWorker;
use crate::evaluation::IntegrityMetricEvent;
use crate::manager::failure::FailureClass;
use ghostr_engine::representation::TransferIdentity;
use ghostr_net::media_log_identity::MediaLogIdentity;
use ghostr_partial_store::partial_range_completion::IntegrityMismatch;
use log::warn;

impl DeliveryWorker {
    pub(super) async fn finish_finalize_error(
        &mut self,
        identity: &TransferIdentity,
        advertised: Option<&str>,
        error: anyhow::Error,
    ) {
        let source = identity.source().as_str();
        if error.downcast_ref::<IntegrityMismatch>().is_some() {
            self.commands
                .evaluation()
                .integrity(IntegrityMetricEvent::HashMismatch);
            if let Some(digest) = advertised {
                self.quarantine_digest_group(identity, digest).await;
            }
            self.note_failed_attempt(identity.post(), source, FailureClass::Permanent);
            return;
        }
        warn!("Finalize failed for {}", MediaLogIdentity::from_url(source));
        self.note_failed_attempt(identity.post(), source, FailureClass::Transient);
    }

    async fn quarantine_digest_group(&mut self, identity: &TransferIdentity, digest: &str) {
        let posts = self.state.catalog_mut().quarantine_mirror_group(
            identity,
            digest,
            crate::manager::time::unix_time_ms(),
        );
        for post in posts {
            if let Err(error) = self.ctx.store.quarantine(post.as_str()).await {
                warn!(
                    "Integrity quarantine failed for {}: {error:#}",
                    post.as_str()
                );
            }
        }
    }
}
