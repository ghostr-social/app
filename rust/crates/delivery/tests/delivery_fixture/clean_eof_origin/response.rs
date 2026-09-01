use super::request::{self, Request};
use super::{OriginState, BODY};
use core::sync::atomic::Ordering;
use tokio::io::AsyncWriteExt as _;
use tokio::net::{TcpListener, TcpStream};

const HEADERS: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nETag: \"v1\"\r\nConnection: close\r\n\r\n";

pub(super) async fn accept(listener: TcpListener, state: OriginState) {
    while let Ok((socket, _)) = listener.accept().await {
        tokio::spawn(answer(socket, state.clone()));
    }
}

async fn answer(mut socket: TcpStream, state: OriginState) {
    let request = request::read(&mut socket).await;
    socket.write_all(HEADERS).await.ok();
    if matches!(request, Request::Head) {
        return;
    }
    state.gets.fetch_add(1, Ordering::SeqCst);
    if matches!(request, Request::Whole) {
        signal_whole(&state);
        state.release.notified().await;
    }
    socket.write_all(BODY).await.ok();
    socket.shutdown().await.ok();
}

fn signal_whole(state: &OriginState) {
    let signal = state.started.lock().expect("valid test fixture").take();
    if let Some(signal) = signal {
        signal.send(()).ok();
    }
}
