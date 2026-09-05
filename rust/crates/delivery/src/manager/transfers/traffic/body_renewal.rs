use super::TransferTraffic;
use crate::manager::body_renewal::{BodyRenewal, BodyRenewalRequest};
use crate::manager::transfers::InternalEvent;
use core::time::Duration;

impl TransferTraffic {
    async fn pause_body(&mut self, wait: Duration) {
        if self.opened { self.publisher.closed(self.transfer, tokio::time::Instant::now()); }
        self.opened = false;
        tokio::time::sleep(wait).await;
        self.transfer = self.transfer.next_window();
        if let Some(host) = self.host.clone() {
            self.opened = self.publisher.resumed(self.transfer, host, tokio::time::Instant::now());
        }
    }

    pub(super) async fn authorize_body_window(&mut self, through: u64) -> anyhow::Result<()> {
        loop {
            let (reply, answer) = tokio::sync::oneshot::channel();
            let request = BodyRenewalRequest { attempt: self.attempt.clone(), through, reply };
            self.events.send(InternalEvent::BodyRenewal(request))
                .map_err(|_| crate::chunk::body_lease::BodyLeaseDenied)?;
            match answer.await.map_err(|_| crate::chunk::body_lease::BodyLeaseDenied)? {
                BodyRenewal::Granted => return Ok(()),
                BodyRenewal::Denied => return Err(crate::chunk::body_lease::BodyLeaseDenied.into()),
                BodyRenewal::WaitUntil(deadline) => {
                    let wait = deadline.saturating_sub(crate::manager::time::unix_time_ms()).max(1);
                    self.pause_body(Duration::from_millis(wait)).await;
                }
            }
        }
    }
}
