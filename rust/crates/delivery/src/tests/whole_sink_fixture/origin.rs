use std::sync::Arc;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Notify};

pub(crate) struct SplitOrigin {
    pub(crate) url: String,
    pub(crate) prefix_sent: oneshot::Receiver<()>,
    pub(crate) release: Arc<Notify>,
}

pub(crate) async fn split(prefix: &'static [u8], suffix: &'static [u8]) -> SplitOrigin {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let release = Arc::new(Notify::new());
    let body_release = std::sync::Arc::clone(&release);
    let (sent, prefix_sent) = oneshot::channel();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.expect("valid test fixture");
        let mut request = [0; 4096];
        assert!(socket.read(&mut request).await.expect("valid test fixture") > 0);
        socket.write_all(prefix).await.expect("valid test fixture");
        sent.send(()).ok();
        body_release.notified().await;
        socket.write_all(suffix).await.expect("valid test fixture");
    });
    SplitOrigin {
        url: format!("http://{address}/video.mp4"),
        prefix_sent,
        release,
    }
}
