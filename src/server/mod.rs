pub(crate) mod client;
pub(crate) mod reader;
pub(crate) mod writer;

use axum::extract::ws::WebSocket;
use futures::StreamExt;
use uuid::Uuid;

use crate::message::model::client::{MessageBody, MessageBuilder};

pub(crate) async fn spawn_client(websocket: WebSocket) {
    let (writer, reader) = websocket.split();

    let mut client = client::Client::new(reader, writer);
    let mesg = MessageBuilder::default()
        .sender(Uuid::nil())
        .body(MessageBody::Response {
            target: client.id(),
            body: "".into(),
        })
        .build()
        .unwrap();

    let mesg = serde_json::to_string(&mesg).unwrap();
    let _ = client.send(axum::extract::ws::Message::Text(mesg)).await;
    let (rhalf, whalf) = client.create_handles();

    tokio::spawn(rhalf.spawn_handler());
    tokio::spawn(whalf.spawn_handler());
}
