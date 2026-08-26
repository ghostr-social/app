use futures_util::{SinkExt as _, StreamExt as _};
use nostr_sdk::{Event, JsonUtil as _};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

pub(crate) async fn auth_retry_relay(event: Event) -> String {
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
    let mut first_id = None;
    while let Some(Ok(message)) = socket.next().await {
        let Message::Text(payload) = message else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&payload) else {
            continue;
        };
        match value.get(0).and_then(Value::as_str) {
            Some("REQ") => handle_request(&mut socket, &value, &event, &mut first_id).await,
            Some("AUTH") => accept_auth(&mut socket, &value).await,
            _ => {}
        }
    }
}

async fn handle_request(
    socket: &mut tokio_tungstenite::WebSocketStream<TcpStream>,
    message: &Value,
    event: &Event,
    first_id: &mut Option<String>,
) {
    let id = message.get(1).and_then(Value::as_str).unwrap_or_default();
    if first_id.is_none() {
        *first_id = Some(id.to_owned());
        let _ = socket
            .send(Message::Text(r#"["AUTH","test-challenge"]"#.to_owned()))
            .await;
        let closed = format!(r#"["CLOSED","{id}","auth-required: login"]"#);
        let _ = socket.send(Message::Text(closed)).await;
        return;
    }
    let stale = format!(
        r#"["EOSE","{}"]"#,
        first_id.as_deref().expect("valid test fixture")
    );
    let result = format!(r#"["EVENT","{id}",{}]"#, event.as_json());
    let eose = format!(r#"["EOSE","{id}"]"#);
    let _ = socket.send(Message::Text(stale)).await;
    let _ = socket.send(Message::Text(result)).await;
    let _ = socket.send(Message::Text(eose)).await;
}

async fn accept_auth(socket: &mut tokio_tungstenite::WebSocketStream<TcpStream>, message: &Value) {
    let id = message
        .get(1)
        .and_then(|event| event.get("id"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    let _ = socket
        .send(Message::Text(format!(r#"["OK","{id}",true,""]"#)))
        .await;
}
