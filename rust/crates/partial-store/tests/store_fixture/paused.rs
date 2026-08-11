use crate::store_fixture::temp_root;
use ghostr_partial_store::partial_range_store::capacity::{Limits, StoreCapacity};
use ghostr_partial_store::partial_range_store::free_space::FreeSpace;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::sync::{oneshot, Mutex};

pub struct PausedStore {
    pub store: Arc<PartialRangeStore>,
    pub root: PathBuf,
    entered: oneshot::Receiver<()>,
    resume: mpsc::Sender<()>,
}

impl PausedStore {
    pub async fn wait_until_admission(&mut self) {
        (&mut self.entered).await.expect("admission entered");
    }

    pub fn resume(&self) {
        self.resume.send(()).expect("resume admission");
    }
}

struct PausedSpace {
    entered: StdMutex<Option<oneshot::Sender<()>>>,
    resume: StdMutex<mpsc::Receiver<()>>,
}

impl FreeSpace for PausedSpace {
    fn available_bytes(&self, _path: &Path) -> Option<u64> {
        if let Some(entered) = self.entered.lock().expect("entered lock").take() {
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

pub fn paused_store(prefix: &str) -> PausedStore {
    let root = temp_root(prefix);
    let (entered_sender, entered) = oneshot::channel();
    let (resume, resume_receiver) = mpsc::channel();
    let space = Arc::new(PausedSpace {
        entered: StdMutex::new(Some(entered_sender)),
        resume: StdMutex::new(resume_receiver),
    });
    let capacity = capacity(space);
    let store = PartialRangeStore::with_capacity(root.clone(), Arc::new(Mutex::new(0)), capacity);
    PausedStore {
        store: Arc::new(store),
        root,
        entered,
        resume,
    }
}

fn capacity(space: Arc<PausedSpace>) -> StoreCapacity {
    let limits = Limits {
        budget: u64::MAX,
        reserve: 0,
    };
    StoreCapacity::new(limits, space, Duration::from_secs(60))
}
