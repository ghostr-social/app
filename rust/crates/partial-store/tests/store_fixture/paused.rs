#![allow(dead_code)]

use super::store_fixture::temp_root;
use crate::partial_range_store::capacity::{Limits, StoreCapacity};
use crate::partial_range_store::free_space::FreeSpace;
use crate::partial_range_store::PartialRangeStore;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex as StdMutex};
use tokio::sync::{oneshot, Mutex};

pub(super) struct PausedStore {
    pub(super) store: Arc<PartialRangeStore>,
    pub(super) root: PathBuf,
    entered: oneshot::Receiver<()>,
    resume: mpsc::Sender<()>,
}

impl PausedStore {
    pub(super) async fn wait_until_admission(&mut self) {
        (&mut self.entered).await.expect("admission entered");
    }

    pub(super) fn resume(&self) {
        self.resume.send(()).expect("resume admission");
    }
}

struct PausedSpace {
    entered: StdMutex<Option<oneshot::Sender<()>>>,
    resume: StdMutex<mpsc::Receiver<()>>,
    pause_after: usize,
    calls: AtomicUsize,
}

impl FreeSpace for PausedSpace {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        let should_pause = self.calls.fetch_add(1, Ordering::SeqCst) == self.pause_after;
        if should_pause {
            let entered = self.entered.lock().expect("entered lock").take();
            let Some(entered) = entered else {
                return Some(1_000);
            };
            let _ = entered.send(());
            self.resume
                .lock()
                .expect("resume lock")
                .recv()
                .expect("resume signal");
        }
        Some(1_000)
    }
}

pub(super) fn paused_store(prefix: &str) -> PausedStore {
    build_store(prefix, 0, Duration::from_secs(60), u64::MAX)
}

pub(super) fn paused_store_after(prefix: &str, pause_after: usize) -> PausedStore {
    build_store(prefix, pause_after, Duration::ZERO, u64::MAX)
}

pub(super) fn paused_store_with_budget(prefix: &str, budget: u64) -> PausedStore {
    build_store(prefix, 0, Duration::from_secs(60), budget)
}

fn build_store(prefix: &str, pause_after: usize, recheck: Duration, budget: u64) -> PausedStore {
    let root = temp_root(prefix);
    let (entered_sender, entered) = oneshot::channel();
    let (resume, resume_receiver) = mpsc::channel();
    let space = Arc::new(PausedSpace {
        entered: StdMutex::new(Some(entered_sender)),
        resume: StdMutex::new(resume_receiver),
        pause_after,
        calls: AtomicUsize::new(0),
    });
    let capacity = capacity(space, recheck, budget);
    let store = PartialRangeStore::with_capacity(root.clone(), Arc::new(Mutex::new(0)), capacity);
    PausedStore {
        store: Arc::new(store),
        root,
        entered,
        resume,
    }
}

fn capacity(space: Arc<PausedSpace>, recheck: Duration, budget: u64) -> StoreCapacity {
    let limits = Limits { budget, reserve: 0 };
    StoreCapacity::new(limits, space, recheck)
}
