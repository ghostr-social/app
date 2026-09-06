use std::sync::mpsc;

pub fn hold_blocking_worker() -> mpsc::Sender<()> {
    let (release, wait) = mpsc::channel();
    tokio::task::spawn_blocking(move || {
        let _ = wait.recv();
    });
    release
}
