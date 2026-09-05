use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};

pub(super) async fn accept_body(
    listener: &TcpListener,
    head_response: &[u8],
) -> (TcpStream, Vec<u8>) {
    loop {
        let (mut socket, _) = listener.accept().await.expect("origin request");
        let mut request = vec![0; 4096];
        let length = socket.read(&mut request).await.expect("request headers");
        assert!(length > 0, "origin receives a request");
        request.truncate(length);
        if !request.starts_with(b"HEAD ") {
            return (socket, request);
        }
        socket
            .write_all(head_response)
            .await
            .expect("HEAD response");
    }
}
