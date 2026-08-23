//! Range-blind origin whose recovery GET pauses after response headers.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Notify};

pub const BODY: &[u8] = b"0123456789abcdef";

pub struct CleanEofOrigin {
    url: String,
    whole_started: Option<oneshot::Receiver<()>>,
    release: Arc<Notify>,
    gets: Arc<AtomicUsize>,
}

pub async fn serve() -> CleanEofOrigin {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let release = Arc::new(Notify::new());
    let gets = Arc::new(AtomicUsize::new(0));
    let (started, whole_started) = oneshot::channel();
    let started = Arc::new(Mutex::new(Some(started)));
    tokio::spawn(accept(listener, gets.clone(), started, release.clone()));
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
            std::time::Duration::from_secs(1),
            self.whole_started.take().unwrap(),
        )
        .await
        .expect("whole GET starts")
        .unwrap();
    }

    pub fn release(&self) {
        self.release.notify_one();
    }

    pub fn gets(&self) -> usize {
        self.gets.load(Ordering::SeqCst)
    }
}

async fn accept(
    listener: TcpListener,
    gets: Arc<AtomicUsize>,
    started: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    release: Arc<Notify>,
) {
    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(answer(
            socket,
            gets.clone(),
            started.clone(),
            release.clone(),
        ));
    }
}

async fn answer(
    mut socket: TcpStream,
    gets: Arc<AtomicUsize>,
    started: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    release: Arc<Notify>,
) {
    let mut request = [0; 4096];
    let read = socket.read(&mut request).await.unwrap_or(0);
    let head = request[..read].starts_with(b"HEAD ");
    let range = request[..read].windows(8).any(|part| part == b"\r\nrange:");
    let response =
        b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nETag: \"v1\"\r\nConnection: close\r\n\r\n";
    socket.write_all(response).await.ok();
    if head {
        return;
    }
    gets.fetch_add(1, Ordering::SeqCst);
    if !range {
        if let Some(signal) = started.lock().unwrap().take() {
            signal.send(()).ok();
        }
        release.notified().await;
    }
    socket.write_all(BODY).await.ok();
    socket.shutdown().await.ok();
}
