//! Range-blind origin whose recovery GET pauses after response headers.

use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Notify};

mod request;
mod response;

pub const BODY: &[u8] = b"0123456789abcdef";

#[derive(Clone)]
pub(super) struct OriginState {
    pub(super) gets: Arc<AtomicUsize>,
    pub(super) started: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    pub(super) release: Arc<Notify>,
}

pub struct CleanEofOrigin {
    url: String,
    whole_started: Option<oneshot::Receiver<()>>,
    release: Arc<Notify>,
    gets: Arc<AtomicUsize>,
}

pub async fn serve() -> CleanEofOrigin {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let release = Arc::new(Notify::new());
    let gets = Arc::new(AtomicUsize::new(0));
    let (started, whole_started) = oneshot::channel();
    let started = Arc::new(Mutex::new(Some(started)));
    let state = OriginState {
        gets: Arc::clone(&gets),
        started,
        release: Arc::clone(&release),
    };
    tokio::spawn(response::accept(listener, state));
    CleanEofOrigin {
        url: format!("http://{address}/video.mp4"),
        whole_started: Some(whole_started),
        release,
        gets,
    }
}

impl CleanEofOrigin {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub async fn wait_whole_started(&mut self) {
        tokio::time::timeout(
            core::time::Duration::from_secs(30),
            self.whole_started.take().expect("valid test fixture"),
        )
        .await
        .expect("whole GET starts")
        .expect("valid test fixture");
    }

    pub fn release(&self) {
        self.release.notify_one();
    }

    pub fn gets(&self) -> usize {
        self.gets.load(Ordering::SeqCst)
    }
}
