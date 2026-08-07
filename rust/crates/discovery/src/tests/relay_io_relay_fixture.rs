use futures_util::{SinkExt, StreamExt};
use nostr_sdk::{Event, JsonUtil};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

pub async fn relay_serving(event: Event) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("relay");
    let url = format!("ws://{}", listener.local_addr().expect("relay address"));
    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(serve_client(stream, event.clone()));
        }
    });
    url
}

async fn serve_client(stream: TcpStream, event: Event) {
    let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
        return;
    };
    while let Some(Ok(message)) = socket.next().await {
        match message {
            Message::Text(payload) => handle_text(&mut socket, &payload, &event).await,
            Message::Ping(bytes) => {
                let _ = socket.send(Message::Pong(bytes)).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}

async fn handle_text<S>(socket: &mut S, payload: &str, event: &Event)
where
    S: futures_util::Sink<Message> + Unpin,
{
    let Ok(message) = serde_json::from_str::<Value>(payload) else {
        return;
    };
    match message.get(0).and_then(Value::as_str) {
        Some("REQ") => send_query_result(socket, &message, event).await,
        Some("EVENT") => send_acceptance(socket, &message).await,
        _ => {}
    }
}

async fn send_query_result<S>(socket: &mut S, message: &Value, event: &Event)
where
    S: futures_util::Sink<Message> + Unpin,
{
    let subscription = message.get(1).and_then(Value::as_str).unwrap_or_default();
    let event_message = format!(r#"["EVENT","{subscription}",{}]"#, event.as_json());
    let eose = format!(r#"["EOSE","{subscription}"]"#);
    let _ = socket.send(Message::Text(event_message)).await;
    tokio::task::yield_now().await;
    let _ = socket.send(Message::Text(eose)).await;
}

async fn send_acceptance<S>(socket: &mut S, message: &Value)
where
    S: futures_util::Sink<Message> + Unpin,
{
    let id = message
        .get(1)
        .and_then(|event| event.get("id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let reply = format!(r#"["OK","{id}",true,""]"#);
    let _ = socket.send(Message::Text(reply)).await;
}
