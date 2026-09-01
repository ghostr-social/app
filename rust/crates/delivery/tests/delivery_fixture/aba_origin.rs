//! Controlled request-aware origin for stale-completion scenarios.

use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use std::sync::Arc;
use tokio::io::AsyncWriteExt as _;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

mod request;
mod response;
use response::{is_head, write_body_headers, write_head};

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
            tokio::spawn(answer(
                socket,
                std::sync::Arc::clone(&bytes),
                server_control.clone(),
            ));
        }
    });
    (format!("http://{address}/video.mp4"), control)
}

async fn answer(mut socket: TcpStream, bytes: Arc<Vec<u8>>, gate: AbaOrigin) {
    let mut request = request::read_line(&mut socket).await;
    if is_head(&request) {
        request::complete_headers(&mut socket, &mut request).await;
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
    request::complete_headers(&mut socket, &mut request).await;
    let range = write_body_headers(&mut socket, &request, bytes.len()).await;
    if attempt == 1 {
        core::future::pending::<()>().await;
    }
    gate.bodies.acquire().await.expect("body gate").forget();
    let _ = socket.write_all(&bytes[range]).await;
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
