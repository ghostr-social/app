use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

pub(crate) async fn auth_failure_before_closed_relay() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("relay");
    let url = format!("ws://{}", listener.local_addr().expect("relay address"));
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(serve_client(stream));
        }
    });
    url
}

pub(crate) async fn auth_closed_then_stale_eose_relay() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("relay");
    let url = format!("ws://{}", listener.local_addr().expect("relay address"));
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(serve_stale_eose_client(stream));
        }
    });
    url
}

async fn serve_client(stream: TcpStream) {
    let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
        return;
    };
    while let Some(message) = socket.next().await {
        match message {
            Ok(Message::Text(payload)) => handle_request(&mut socket, payload).await,
            Ok(Message::Ping(bytes)) => {
                let _ = socket.send(Message::Pong(bytes)).await;
            }
            Ok(Message::Close(_)) | Err(_) => return,
            _ => {}
        }
    }
}

async fn serve_stale_eose_client(stream: TcpStream) {
    let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
        return;
    };
    while let Some(Ok(Message::Text(payload))) = socket.next().await {
        let Ok(message) = serde_json::from_str::<Value>(&payload) else {
            continue;
        };
        let Some(id) = request_id(&message) else {
            continue;
        };
        let closed = format!(r#"["CLOSED","{id}","auth-required: login"]"#);
        let eose = format!(r#"["EOSE","{id}"]"#);
        let _ = socket.send(Message::Text(closed)).await;
        let _ = socket.send(Message::Text(eose)).await;
    }
}

async fn handle_request(
    socket: &mut tokio_tungstenite::WebSocketStream<TcpStream>,
    payload: String,
) {
    let Ok(message) = serde_json::from_str::<Value>(&payload) else {
        return;
    };
    let Some(id) = request_id(&message) else {
        return;
    };
    let _ = socket
        .send(Message::Text(r#"["AUTH","test-challenge"]"#.to_owned()))
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    let closed = format!(r#"["CLOSED","{id}","auth-required: login"]"#);
    let _ = socket.send(Message::Text(closed)).await;
}

fn request_id(message: &Value) -> Option<&str> {
    (message.get(0)?.as_str()? == "REQ").then(|| message.get(1)?.as_str())?
}
