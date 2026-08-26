use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use tokio::io::AsyncReadExt as _;
use tokio::net::TcpListener;

struct LocalClient(Client);

impl MediaHttpRequests for LocalClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

pub(super) async fn fixture() -> (MediaRequestExecutor, String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let url = format!(
        "http://{}/asset.ts",
        listener.local_addr().expect("listener address")
    );
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("request");
        let mut request = [0; 1024];
        assert!(socket.read(&mut request).await.expect("read") > 0);
        core::future::pending::<()>().await;
    });
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("client");
    let limits = MediaRequestLimits::try_new(1, 1).expect("positive test limits");
    let executor = MediaRequestExecutor::new(Arc::new(LocalClient(client)), limits);
    (executor, url, server)
}
