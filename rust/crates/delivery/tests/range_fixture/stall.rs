//! Raw TCP fixture that answers one body request coherently, sends only
//! `prefix`, then stalls forever.

use core::ops::Range;
use request::Request;
use tokio::io::AsyncWriteExt as _;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

mod request;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BodyKind {
    Whole,
    Range(Range<u64>),
}

pub async fn serve_stalling(prefix: Vec<u8>, total: u64) -> String {
    serve_stalling_signaled(prefix, total).await.0
}

pub async fn serve_stalling_signaled(
    prefix: Vec<u8>,
    total: u64,
) -> (String, oneshot::Receiver<BodyKind>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind stall fixture");
    let address = listener.local_addr().expect("stall address");
    let (started, observed) = oneshot::channel();
    tokio::spawn(serve(listener, prefix, total, started));
    (format!("http://{address}/video.mp4"), observed)
}

async fn serve(
    listener: TcpListener,
    prefix: Vec<u8>,
    total: u64,
    started: oneshot::Sender<BodyKind>,
) {
    loop {
        let (mut socket, _) = listener.accept().await.expect("stall accept");
        match request::read(&mut socket, total).await {
            Request::Head => request::write_probe(&mut socket, total).await,
            Request::Body(body) => {
                started.send(body.kind()).ok();
                stall(&mut socket, &prefix, total, body).await;
                return;
            }
        }
    }
}

async fn stall(socket: &mut TcpStream, prefix: &[u8], total: u64, body: request::BodyRequest) {
    body.write_response(socket, total).await;
    let range = body.range(total);
    let requested = range.end - range.start;
    let delivered = prefix
        .len()
        .min(usize::try_from(requested).unwrap_or(usize::MAX));
    let _ = socket.write_all(&prefix[..delivered]).await;
    let _ = socket.flush().await;
    core::future::pending::<()>().await;
}
