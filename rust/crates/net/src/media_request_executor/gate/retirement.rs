use super::{MediaRequestGate, RequestLease};
use crate::internet_allowance::InternetReservation;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::RequestAuthority;

struct Retirement {
    body: Option<InternetReservation>,
    gate: MediaRequestGate,
    authority: RequestAuthority,
    priority: PreemptionAuthority,
    armed: bool,
}

impl RequestLease {
    /// The slot remains occupied until durable settlement, bounding pending IO.
    pub(super) fn defer_retirement(&mut self) -> bool {
        let Some(body) = self.body.take() else {
            return false;
        };
        let work = Retirement {
            body: Some(body),
            gate: self.gate.clone(),
            authority: self.authority.clone(),
            priority: self.priority,
            armed: self.armed,
        };
        match tokio::runtime::Handle::try_current() {
            Ok(runtime) => {
                runtime.spawn_blocking(move || drop(work));
            }
            Err(_) => drop(work),
        }
        true
    }
}

impl Drop for Retirement {
    fn drop(&mut self) {
        drop(self.body.take());
        if self.armed {
            self.gate.release(&self.authority, self.priority);
        }
    }
}
