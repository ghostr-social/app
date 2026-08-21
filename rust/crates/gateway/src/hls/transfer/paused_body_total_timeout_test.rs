use super::HlsTransfer;
use crate::hls::asset_response::AssetResponseEnvelope;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
use reqwest::{Client, RequestBuilder};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

struct LocalClient(Client);

impl MediaHttpRequests for LocalClient {
    fn get(&self, url: &str) -> anyhow::Result<RequestBuilder> {
        Ok(self.0.get(url))
    }
}

#[tokio::test]
async fn an_unpolled_player_body_releases_its_lease_at_the_total_deadline() {
    let (executor, url, second_hit, server) = fixture().await;
    let request = executor
        .get(&url, PreemptionAuthority::PlaybackCritical)
        .expect("first request");
    let timeouts = HlsTransferTimeouts::new(
        Duration::from_secs(1),
        Duration::from_secs(1),
        Duration::from_millis(40),
    );
    let transfer = HlsTransfer::open(request, timeouts).await.expect("open");
    let proxy = transfer
        .into_proxy(AssetResponseEnvelope::Full { length: Some(8) })
        .expect("proxy");
    let _unpolled_body = proxy.into_body();
    let next = spawn_next(executor, url);

    tokio::time::timeout(Duration::from_millis(250), second_hit)
        .await
        .expect("total deadline releases upstream lease")
        .expect("second request signal");
    next.abort();
    server.abort();
}

async fn fixture() -> (
    MediaRequestExecutor,
    String,
    oneshot::Receiver<()>,
    tokio::task::JoinHandle<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
    let url = format!("http://{}/asset.ts", listener.local_addr().unwrap());
    let (second_hit, observed) = oneshot::channel();
    let server = tokio::spawn(serve(listener, second_hit));
    let client = Client::builder().no_proxy().build().expect("client");
    let limits = MediaRequestLimits::try_new(1, 1).unwrap();
    let executor = MediaRequestExecutor::new(Arc::new(LocalClient(client)), limits);
    (executor, url, observed, server)
}

async fn serve(listener: TcpListener, second_hit: oneshot::Sender<()>) {
    let (first, _) = listener.accept().await.expect("first request");
    tokio::spawn(stall_body(first));
    let (_second, _) = listener.accept().await.expect("second request");
    let _ = second_hit.send(());
}

async fn stall_body(mut socket: TcpStream) {
    let mut request = [0; 1024];
    assert!(socket.read(&mut request).await.expect("read request") > 0);
    socket
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\n\r\n1234")
        .await
        .expect("write response");
    std::future::pending::<()>().await;
}

fn spawn_next(
    executor: MediaRequestExecutor,
    url: String,
) -> tokio::task::JoinHandle<anyhow::Result<()>> {
    tokio::spawn(async move {
        let request = executor.get(&url, PreemptionAuthority::Transition)?;
        let _response = request.admit().await?.send().await?;
        Ok(())
    })
}
