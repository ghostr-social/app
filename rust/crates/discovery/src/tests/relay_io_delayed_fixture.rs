use futures_util::{SinkExt, StreamExt};
use nostr_sdk::{Event, JsonUtil};
use serde_json::Value;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

pub(crate) async fn delayed_relay(event: Option<Event>, delay: Duration) -> String {
    result_relay(event, delay, true).await
}

pub(crate) async fn incomplete_relay(event: Event) -> String {
    result_relay(Some(event), Duration::ZERO, false).await
}

async fn result_relay(event: Option<Event>, delay: Duration, complete: bool) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("relay");
    let url = format!("ws://{}", listener.local_addr().expect("relay address"));
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let event = event.clone();
            tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                serve_client(stream, event, complete).await;
            });
        }
    });
    url
}

async fn serve_client(stream: TcpStream, event: Option<Event>, complete: bool) {
    let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
        return;
    };
    while let Some(Ok(message)) = socket.next().await {
        match message {
            Message::Text(payload) => {
                send_result(&mut socket, &payload, event.as_ref(), complete).await
            }
            Message::Ping(bytes) => {
                let _ = socket.send(Message::Pong(bytes)).await;
            }
            Message::Close(_) => return,
            _ => {}
        }
    }
}

async fn send_result<S>(socket: &mut S, payload: &str, event: Option<&Event>, complete: bool)
where
    S: futures_util::Sink<Message> + Unpin,
{
    let Ok(message) = serde_json::from_str::<Value>(payload) else {
        return;
    };
    if message.get(0).and_then(Value::as_str) != Some("REQ") {
        return;
    }
    let id = message.get(1).and_then(Value::as_str).unwrap_or_default();
    if let Some(event) = event {
        let frame = format!(r#"["EVENT","{id}",{}]"#, event.as_json());
        let _ = socket.send(Message::Text(frame)).await;
    }
    if complete {
        let _ = socket
            .send(Message::Text(format!(r#"["EOSE","{id}"]"#)))
            .await;
    }
}
