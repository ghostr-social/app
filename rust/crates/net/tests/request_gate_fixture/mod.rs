#![allow(dead_code)]

use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Semaphore};

pub mod request;

pub struct LocalMediaClient(Client);

impl LocalMediaClient {
    pub fn shared() -> Arc<dyn MediaHttpRequests> {
        Arc::new(Self(
            Client::builder()
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("local media client"),
        ))
    }
}

impl MediaHttpRequests for LocalMediaClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

pub struct HeldOrigin {
    pub url: String,
    hits: mpsc::UnboundedReceiver<()>,
    release: Arc<Semaphore>,
}

impl HeldOrigin {
    pub async fn serve() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (hits, hit_events) = mpsc::unbounded_channel();
        let release = Arc::new(Semaphore::new(0));
        let task_release = Arc::clone(&release);
        tokio::spawn(async move {
            while let Ok((socket, _)) = listener.accept().await {
                tokio::spawn(answer(socket, hits.clone(), Arc::clone(&task_release)));
            }
        });
        Self {
            url: format!("http://{address}/media"),
            hits: hit_events,
            release,
        }
    }

    pub async fn expect_hit(&mut self) {
        tokio::time::timeout(Duration::from_secs(1), self.hits.recv())
            .await
            .expect("request reaches origin")
            .expect("origin hit channel");
    }

    pub async fn expect_quiet(&mut self) {
        assert!(
            tokio::time::timeout(Duration::from_millis(75), self.hits.recv())
                .await
                .is_err(),
            "blocked request reached its origin"
        );
    }

    pub fn release_one(&self) {
        self.release.add_permits(1);
    }
}

async fn answer(mut socket: TcpStream, hits: mpsc::UnboundedSender<()>, release: Arc<Semaphore>) {
    let mut request = [0u8; 4096];
    let _ = socket.read(&mut request).await;
    let _ = hits.send(());
    let _ = socket
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1\r\nConnection: close\r\n\r\n")
        .await;
    let _ = release.acquire().await;
    let _ = socket.write_all(b"x").await;
}
