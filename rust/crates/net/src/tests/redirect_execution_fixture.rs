use crate::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use crate::outbound_media_client::{MediaHttpClient, MediaHttpRequests};
use anyhow::{Context, Result};
use ghostr_engine::RequestAuthority;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

struct InitialOriginClient {
    initial: RequestAuthority,
    local: Client,
    guarded: MediaHttpClient,
}

impl MediaHttpRequests for InitialOriginClient {
    fn get(&self, raw_url: &str) -> Result<RequestBuilder> {
        let authority = RequestAuthority::from_url(raw_url).context("request authority")?;
        if authority == self.initial {
            return Ok(self.local.get(raw_url));
        }
        self.guarded.get(raw_url)
    }
}

pub async fn redirected(location: &str) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let location = location.to_owned();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.unwrap() > 0);
        let response =
            format!("HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\n\r\n");
        socket.write_all(response.as_bytes()).await.unwrap();
    });
    (format!("http://{address}/video.mp4"), server)
}

pub fn executor(initial: &str) -> MediaRequestExecutor {
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let source = InitialOriginClient {
        initial: RequestAuthority::from_url(initial).unwrap(),
        local: client,
        guarded: MediaHttpClient::public().unwrap(),
    };
    MediaRequestExecutor::new(Arc::new(source), MediaRequestLimits::try_new(1, 1).unwrap())
}
