use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub async fn redirect_chain(redirects: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            tokio::spawn(answer(socket, redirects));
        }
    });
    format!("http://{address}/0")
}

async fn answer(mut socket: TcpStream, redirects: usize) {
    let mut request = [0u8; 2048];
    let read = socket.read(&mut request).await.unwrap_or(0);
    let line = String::from_utf8_lossy(&request[..read]);
    let step = line
        .split_whitespace()
        .nth(1)
        .and_then(|path| path.trim_start_matches('/').parse::<usize>().ok())
        .unwrap_or(0);
    let response = if step < redirects {
        format!(
            "HTTP/1.1 302 Found\r\nLocation: /{}\r\nContent-Length: 0\r\n\r\n",
            step + 1
        )
    } else {
        "HTTP/1.1 200 OK\r\nContent-Length: 1\r\n\r\nx".to_owned()
    };
    let _ = socket.write_all(response.as_bytes()).await;
}
