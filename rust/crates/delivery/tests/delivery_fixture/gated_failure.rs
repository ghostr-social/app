use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Notify};
use tokio::time::Instant;

pub struct GatedFailure {
    url: String,
    attempts: Arc<AtomicUsize>,
    started: Option<oneshot::Receiver<()>>,
    release: Arc<Notify>,
}

pub async fn serve() -> GatedFailure {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind gate");
    let address = listener.local_addr().expect("gate address");
    let attempts = Arc::new(AtomicUsize::new(0));
    let release = Arc::new(Notify::new());
    let (started, observed) = oneshot::channel();
    let started = Arc::new(Mutex::new(Some(started)));
    tokio::spawn(accept(listener, attempts.clone(), release.clone(), started));
    GatedFailure {
        url: format!("http://{address}/video.mp4"),
        attempts,
        started: Some(observed),
        release,
    }
}

impl GatedFailure {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn attempts(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }

    pub async fn wait_started(&mut self) {
        self.started
            .take()
            .expect("first wait")
            .await
            .expect("start signal");
    }

    pub fn release(&self) {
        self.release.notify_one();
    }

    pub async fn wait_for_attempts(&self, expected: usize) {
        let deadline = Instant::now() + Duration::from_millis(10_000);
        while self.attempts() < expected {
            assert!(Instant::now() < deadline, "retry did not start");
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }
}

async fn accept(
    listener: TcpListener,
    attempts: Arc<AtomicUsize>,
    release: Arc<Notify>,
    started: Arc<Mutex<Option<oneshot::Sender<()>>>>,
) {
    while let Ok((socket, _)) = listener.accept().await {
        let attempts = attempts.clone();
        let started = started.clone();
        let release = release.clone();
        tokio::spawn(async move { answer(socket, attempts, started, release).await });
    }
}

async fn answer(
    mut socket: TcpStream,
    attempts: Arc<AtomicUsize>,
    started: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    release: Arc<Notify>,
) {
    let mut request = [0; 4096];
    let read = socket.read(&mut request).await.unwrap_or(0);
    if request[..read].starts_with(b"HEAD ") {
        attempts.fetch_add(1, Ordering::SeqCst);
        let signal = started.lock().expect("started").take();
        if let Some(signal) = signal {
            signal.send(()).ok();
            release.notified().await;
        }
    }
    let response = b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n";
    socket.write_all(response).await.ok();
}
