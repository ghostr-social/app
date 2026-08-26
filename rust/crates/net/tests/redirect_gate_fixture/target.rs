use core::time::Duration;
use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Semaphore};

pub struct TargetOrigin {
    pub held_url: String,
    pub redirected_url: String,
    pub delayed_url: String,
    hits: mpsc::UnboundedReceiver<String>,
    release: Arc<Semaphore>,
}

impl TargetOrigin {
    pub async fn serve() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("valid test fixture");
        let address = listener.local_addr().expect("valid test fixture");
        let (events, hits) = mpsc::unbounded_channel();
        let release = Arc::new(Semaphore::new(0));
        spawn(listener, events, Arc::clone(&release));
        Self {
            held_url: format!("http://{address}/held"),
            redirected_url: format!("http://{address}/redirected"),
            delayed_url: format!("http://{address}/delayed"),
            hits,
            release,
        }
    }

    pub async fn hit(&mut self) -> String {
        tokio::time::timeout(Duration::from_secs(1), self.hits.recv())
            .await
            .expect("target hit")
            .expect("target event")
    }

    pub async fn quiet(&mut self) {
        assert!(
            tokio::time::timeout(Duration::from_millis(75), self.hits.recv())
                .await
                .is_err(),
            "redirect bypassed target authority admission"
        );
    }

    pub fn release(&self) {
        self.release.add_permits(1);
    }
}

fn spawn(listener: TcpListener, events: mpsc::UnboundedSender<String>, release: Arc<Semaphore>) {
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            tokio::spawn(answer(socket, events.clone(), Arc::clone(&release)));
        }
    });
}

async fn answer(
    mut socket: TcpStream,
    hits: mpsc::UnboundedSender<String>,
    release: Arc<Semaphore>,
) {
    let mut request = [0u8; 4096];
    let read = socket.read(&mut request).await.unwrap_or(0);
    let text = String::from_utf8_lossy(&request[..read]).to_string();
    let _ = hits.send(text.clone());
    if text.starts_with("GET /delayed ") {
        tokio::time::sleep(Duration::from_millis(40)).await;
    }
    let _ = socket
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\n")
        .await;
    if text.starts_with("GET /held ") {
        let _ = release.acquire().await;
    }
    let _ = socket.write_all(b"x").await;
}
