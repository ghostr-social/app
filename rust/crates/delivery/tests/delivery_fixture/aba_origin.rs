//! Controlled range origin for an A→B→A stale-completion scenario.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

mod response;
use response::{is_head, write_head, write_range_headers};

#[derive(Clone)]
pub struct AbaOrigin {
    hits: Arc<AtomicUsize>,
    first_headers: Arc<Semaphore>,
    bodies: Arc<Semaphore>,
}

pub async fn serve(bytes: Vec<u8>) -> (String, AbaOrigin) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ABA origin");
    let address = listener.local_addr().expect("ABA origin address");
    let control = AbaOrigin {
        hits: Arc::new(AtomicUsize::new(0)),
        first_headers: Arc::new(Semaphore::new(0)),
        bodies: Arc::new(Semaphore::new(0)),
    };
    let server_control = control.clone();
    let bytes = Arc::new(bytes);
    tokio::spawn(async move {
        loop {
            let (socket, _) = listener.accept().await.expect("accept ABA request");
            tokio::spawn(answer(socket, bytes.clone(), server_control.clone()));
        }
    });
    (format!("http://{address}/video.mp4"), control)
}

async fn answer(mut socket: TcpStream, bytes: Arc<Vec<u8>>, gate: AbaOrigin) {
    let mut request = [0u8; 4096];
    let read = socket.read(&mut request).await.unwrap_or(0);
    if is_head(&request[..read]) {
        write_head(&mut socket, bytes.len()).await;
        return;
    }
    let attempt = gate.hits.fetch_add(1, Ordering::SeqCst) + 1;
    if attempt == 1 {
        gate.first_headers
            .acquire()
            .await
            .expect("first gate")
            .forget();
    }
    write_range_headers(&mut socket, bytes.len()).await;
    if attempt == 1 {
        std::future::pending::<()>().await;
    }
    gate.bodies.acquire().await.expect("body gate").forget();
    let _ = socket.write_all(&bytes).await;
    let _ = socket.shutdown().await;
}

impl AbaOrigin {
    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }

    pub fn release_first_headers(&self) {
        self.first_headers.add_permits(1);
    }

    pub fn release_body(&self) {
        self.bodies.add_permits(1);
    }

    pub async fn wait_for_hits(&self, expected: usize) {
        let waiting = async {
            while self.hits() < expected {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        };
        tokio::time::timeout(Duration::from_secs(2), waiting)
            .await
            .expect("timed out waiting for ABA origin");
    }
}
