//! Occupy Tokio's sole blocking worker to model unrelated filesystem load.

use std::sync::mpsc;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub(super) fn runtime() -> Runtime {
    Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build()
        .expect("single filesystem worker runtime")
}

pub(super) struct BusyDisk {
    release: mpsc::Sender<()>,
    task: JoinHandle<()>,
}

impl BusyDisk {
    pub(super) async fn occupy() -> Self {
        let (release, wait) = mpsc::channel();
        let (started, ready) = oneshot::channel();
        let task = tokio::task::spawn_blocking(move || {
            started.send(()).expect("disk blocker listener");
            let _release_or_owner_dropped = wait.recv();
        });
        ready.await.expect("disk worker occupied");
        Self { release, task }
    }

    pub(super) async fn finish(self) {
        self.release.send(()).expect("release disk worker");
        self.task.await.expect("disk worker finished");
    }
}
