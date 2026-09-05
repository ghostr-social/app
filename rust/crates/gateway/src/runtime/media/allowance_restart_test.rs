use super::executor;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::internet_allowance::InternetDataLimit;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

#[tokio::test]
async fn runtime_executor_keeps_spent_allowance_across_restart() -> anyhow::Result<()> {
    let root = std::env::temp_dir().join(format!("warp-runtime-allowance-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    let origin = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("http://{}/video", origin.local_addr()?);
    let server = tokio::spawn(async move {
        let (mut socket, _) = origin.accept().await?;
        let read = socket.read(&mut [0; 4096]).await?;
        assert!(read > 0, "origin receives the request");
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 16\r\n\r\n0123456789abcdef")
            .await
    });
    let requests = executor(
        Arc::new(LocalClient),
        2,
        &root,
        InternetDataLimit::Bytes(16),
    )?;
    let admitted = requests
        .get(&url, PreemptionAuthority::PlaybackCritical)?
        .body_limit(16)
        .admit()
        .await?;
    let mut response = admitted
        .send_with_redirect_deadline(
            tokio::time::Instant::now() + core::time::Duration::from_secs(2),
        )
        .await?;
    while response.chunk().await?.is_some() {}
    server.await??;
    drop(response);
    drop(requests);
    let restarted = executor(
        Arc::new(LocalClient),
        2,
        &root,
        InternetDataLimit::Bytes(16),
    )?;
    let denied = restarted
        .get(&url, PreemptionAuthority::PlaybackCritical)?
        .body_limit(1)
        .admit()
        .await;
    assert!(
        denied.is_err(),
        "restarting must preserve spent Internet allowance"
    );
    drop(restarted);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

struct LocalClient;
impl MediaHttpRequests for LocalClient {
    fn get(&self, url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        Ok(reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .build()?
            .get(url))
    }
}
