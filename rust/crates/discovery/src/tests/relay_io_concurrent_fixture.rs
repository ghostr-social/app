use futures_util::{SinkExt as _, StreamExt as _};
use nostr_sdk::{Event, JsonUtil as _};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
pub(crate) async fn relay_closing_empty_before_event(event: Event) -> String {
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
    let mut video = None;
    let mut empty = None;
    while let Some(payload) = next_text(&mut socket).await {
        let Ok(message) = serde_json::from_str::<Value>(&payload) else {
            continue;
        };
        let Some(id) = request_id(&message) else {
            continue;
        };
        if requests_kind(&message, 22) {
            video = Some(id);
        } else {
            empty = Some(id);
        }
        if video.is_some() && empty.is_some() {
            let video = video.take().expect("video request");
            let empty = empty.take().expect("empty request");
            send_results(&mut socket, &event, &video, &empty).await;
        }
    }
}

async fn send_results(
    socket: &mut WebSocketStream<TcpStream>,
    event: &Event,
    video: &str,
    empty: &str,
) {
    let _ = socket
        .send(Message::Text(format!(r#"["EOSE","{empty}"]"#)))
        .await;
    wait_for_close(socket, empty).await;
    let event_message = format!(r#"["EVENT","{video}",{}]"#, event.as_json());
    let _ = socket.send(Message::Text(event_message)).await;
    let _ = socket
        .send(Message::Text(format!(r#"["EOSE","{video}"]"#)))
        .await;
}

async fn wait_for_close(socket: &mut WebSocketStream<TcpStream>, id: &str) {
    while let Some(payload) = next_text(socket).await {
        let Ok(message) = serde_json::from_str::<Value>(&payload) else {
            continue;
        };
        if message.get(0).and_then(Value::as_str) == Some("CLOSE")
            && message.get(1).and_then(Value::as_str) == Some(id)
        {
            return;
        }
    }
}

async fn next_text(socket: &mut WebSocketStream<TcpStream>) -> Option<String> {
    while let Some(Ok(message)) = socket.next().await {
        match message {
            Message::Text(payload) => return Some(payload),
            Message::Ping(bytes) => {
                let _ = socket.send(Message::Pong(bytes)).await;
            }
            Message::Close(_) => return None,
            _ => {}
        }
    }
    None
}

fn request_id(message: &Value) -> Option<String> {
    (message.get(0)?.as_str()? == "REQ").then(|| message.get(1)?.as_str().map(ToOwned::to_owned))?
}

fn requests_kind(message: &Value, kind: u64) -> bool {
    message
        .as_array()
        .into_iter()
        .flatten()
        .skip(2)
        .filter_map(|filter| filter.get("kinds")?.as_array())
        .flatten()
        .any(|value| value.as_u64() == Some(kind))
}
