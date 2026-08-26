#![allow(dead_code)]

use core::time::Duration;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

pub mod chain;
pub mod target;

pub struct OneHopClient(Client);

impl OneHopClient {
    pub fn shared() -> Arc<dyn MediaHttpRequests> {
        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("valid test fixture");
        Arc::new(Self(client))
    }
}

impl MediaHttpRequests for OneHopClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

pub async fn redirect_origin(target: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let target = target.clone();
            tokio::spawn(async move {
                let mut request = [0u8; 2048];
                let _ = socket.read(&mut request).await;
                let response = format!(
                    "HTTP/1.1 302 Found\r\nLocation: {target}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                );
                let _ = socket.write_all(response.as_bytes()).await;
            });
        }
    });
    format!("http://{address}/start")
}

pub async fn delayed_redirect_origin(
    target: String,
    delay: Duration,
) -> (String, oneshot::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let (contacted, contact) = oneshot::channel();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("valid test fixture");
        let mut request = [0u8; 2048];
        let _ = socket.read(&mut request).await;
        let _ = contacted.send(());
        tokio::time::sleep(delay).await;
        let response = format!(
            "HTTP/1.1 302 Found\r\nLocation: {target}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        let _ = socket.write_all(response.as_bytes()).await;
    });
    (format!("http://{address}/start"), contact)
}

pub async fn loop_origin() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let url = format!("http://{address}/loop");
    let target = format!("{url}#next");
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("valid test fixture");
        let mut request = [0u8; 2048];
        let _ = socket.read(&mut request).await;
        let response =
            format!("HTTP/1.1 302 Found\r\nLocation: {target}\r\nContent-Length: 0\r\n\r\n");
        let _ = socket.write_all(response.as_bytes()).await;
    });
    url
}
