use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Notify};

pub(crate) struct SplitOrigin {
    pub url: String,
    pub prefix_sent: oneshot::Receiver<()>,
    pub release: Arc<Notify>,
}

pub(crate) async fn split(prefix: &'static [u8], suffix: &'static [u8]) -> SplitOrigin {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let release = Arc::new(Notify::new());
    let body_release = release.clone();
    let (sent, prefix_sent) = oneshot::channel();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0; 4096];
        assert!(socket.read(&mut request).await.unwrap() > 0);
        socket.write_all(prefix).await.unwrap();
        sent.send(()).ok();
        body_release.notified().await;
        socket.write_all(suffix).await.unwrap();
    });
    SplitOrigin {
        url: format!("http://{address}/video.mp4"),
        prefix_sent,
        release,
    }
}
