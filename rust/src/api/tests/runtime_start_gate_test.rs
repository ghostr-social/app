//! Engine startup remains exclusive across awaits and cancellation.

use crate::api::runtime_registry::StartGate;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::oneshot;

#[tokio::test]
async fn start_permit_blocks_overlap_and_releases_when_cancelled() {
    let gate = Arc::new(StartGate::new());
    let task_gate = gate.clone();
    let (entered, started) = oneshot::channel();
    let task = tokio::spawn(async move {
        let _permit = task_gate.acquire(|| false).expect("first start");
        let _ = entered.send(());
        std::future::pending::<()>().await;
    });
    started.await.expect("start entered an awaited phase");

    assert!(gate.acquire(|| false).is_err());
    task.abort();
    let _ = task.await;
    assert!(gate.acquire(|| false).is_ok());
}

#[test]
fn installed_engine_rejects_another_start() {
    let gate = Arc::new(StartGate::new());
    assert!(gate.acquire(|| true).is_err());
    assert!(gate.acquire(|| false).is_ok());
}

#[test]
fn stale_installed_snapshot_cannot_reenter_after_installation() {
    let gate = Arc::new(StartGate::new());
    let installed = AtomicBool::new(false);
    let first = gate
        .acquire(|| installed.load(Ordering::Acquire))
        .expect("first start");
    let contender_snapshot = installed.load(Ordering::Acquire);
    installed.store(true, Ordering::Release);
    drop(first);

    assert!(!contender_snapshot);
    assert!(gate.acquire(|| installed.load(Ordering::Acquire)).is_err());
}
