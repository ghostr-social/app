use futures_util::{SinkExt as _, StreamExt as _};
use nostr_sdk::{Event, JsonUtil as _};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

pub async fn relay_serving(events: Vec<Event>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("relay");
    let url = format!("ws://{}", listener.local_addr().expect("relay address"));
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(serve_client(stream, events.clone()));
        }
    });
    url
}

async fn serve_client(stream: TcpStream, events: Vec<Event>) {
    let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
        return;
    };
    while let Some(Ok(message)) = socket.next().await {
        match message {
            Message::Text(payload) => answer(&mut socket, &payload, &events).await,
            Message::Ping(bytes) => {
                let _ = socket.send(Message::Pong(bytes)).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

async fn answer<S>(socket: &mut S, payload: &str, events: &[Event])
where
    S: futures_util::Sink<Message> + Unpin,
{
    let Ok(message) = serde_json::from_str::<Value>(payload) else {
        return;
    };
    if message.get(0).and_then(Value::as_str) != Some("REQ") {
        return;
    }
    let subscription = message.get(1).and_then(Value::as_str).unwrap_or_default();
    for event in events {
        let response = format!(r#"["EVENT","{subscription}",{}]"#, event.as_json());
        let _ = socket.send(Message::Text(response)).await;
    }
    let eose = format!(r#"["EOSE","{subscription}"]"#);
    let _ = socket.send(Message::Text(eose)).await;
}
