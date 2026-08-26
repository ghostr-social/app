use crate::relay::io::{RelayBroadcastIo, RelayIo, RelayIoFuture, RelayReadIo, RelayReadResult};
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

pub(crate) struct HealthRelayIo {
    failing: String,
    recovered: AtomicBool,
    reads: Mutex<Vec<Vec<String>>>,
}

impl HealthRelayIo {
    pub(super) fn new(failing: &str) -> Self {
        Self {
            failing: failing.to_owned(),
            recovered: AtomicBool::new(false),
            reads: Mutex::new(Vec::new()),
        }
    }

    pub(super) fn recover(&self) {
        self.recovered.store(true, Ordering::Release);
    }

    pub(super) fn reads(&self) -> Vec<Vec<String>> {
        self.reads.lock().expect("reads").clone()
    }
}

impl RelayIo for HealthRelayIo {
    fn read(&self, mut request: RelayReadIo) -> RelayIoFuture<'_, RelayReadResult> {
        Box::pin(async move {
            self.reads
                .lock()
                .expect("reads")
                .push(request.relays.clone());
            let recovered = self.recovered.load(Ordering::Acquire);
            let failed = failed_relays(&request.relays, &self.failing, recovered);
            let completed = completed_relays(&request.relays, &failed);
            let admissions = request.admissions.as_mut().expect("owner admissions");
            admissions.settle(&completed, &failed);
            Ok(read_result(failed.is_empty()))
        })
    }

    fn broadcast(&self, _request: RelayBroadcastIo) -> RelayIoFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

fn failed_relays(relays: &[String], failing: &str, recovered: bool) -> Vec<String> {
    relays
        .iter()
        .filter(|url| !recovered && url.as_str() == failing)
        .cloned()
        .collect()
}

fn completed_relays(relays: &[String], failed: &[String]) -> Vec<String> {
    relays
        .iter()
        .filter(|url| !failed.contains(url))
        .cloned()
        .collect()
}

fn read_result(complete: bool) -> RelayReadResult {
    if complete {
        RelayReadResult::complete(Vec::new())
    } else {
        RelayReadResult::incomplete(Vec::new())
    }
}
