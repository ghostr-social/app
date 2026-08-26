use super::NetworkProfile;
use core::time::Duration;
use std::sync::RwLock;
use tokio::sync::{Mutex, Notify};
use tokio::time::Instant;

const BITS_PER_BYTE: f64 = 8.0;
const BITS_PER_KILOBIT: f64 = 1_000.0;

#[derive(Debug, Default)]
pub(super) struct SharedBandwidth {
    service: Mutex<()>,
}

impl SharedBandwidth {
    pub(super) async fn pace(
        &self,
        bytes: u64,
        profile: &RwLock<NetworkProfile>,
        changed: &Notify,
    ) {
        let _service = self.service.lock().await;
        serve(bytes as f64 * BITS_PER_BYTE, profile, changed).await;
    }
}

async fn serve(mut remaining: f64, profile: &RwLock<NetworkProfile>, changed: &Notify) {
    while remaining > 0.0 {
        let update = changed.notified();
        tokio::pin!(update);
        update.as_mut().enable();
        let rate = bandwidth(profile);
        if rate == 0 {
            return;
        }
        let started = Instant::now();
        tokio::select! {
            () = tokio::time::sleep(service_time(remaining, rate)) => return,
            () = update => remaining = unserved(remaining, rate, started.elapsed()),
        }
    }
}

fn bandwidth(profile: &RwLock<NetworkProfile>) -> u64 {
    profile
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .bandwidth_kbps
}

fn service_time(bits: f64, bandwidth_kbps: u64) -> Duration {
    Duration::from_secs_f64(bits / (bandwidth_kbps as f64 * BITS_PER_KILOBIT))
}

fn unserved(bits: f64, bandwidth_kbps: u64, elapsed: Duration) -> f64 {
    let served = elapsed.as_secs_f64() * bandwidth_kbps as f64 * BITS_PER_KILOBIT;
    (bits - served).max(0.0)
}
