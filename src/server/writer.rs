use axum::extract::ws::{Message, WebSocket};
use derive_builder::Builder;
use futures::{stream::SplitSink, SinkExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::model::client;

#[cfg_attr(debug_assertions, allow(dead_code))]
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct WriterHalf {
    id: Uuid,
    sink: SplitSink<WebSocket, Message>,
    rx: mpsc::UnboundedReceiver<client::Message>,
}

impl WriterHalf {
    #[tracing::instrument(skip_all, fields(id = self.id.to_string()))]
    pub async fn spawn_writer(mut self) {
        while let Some(result) = self.rx.recv().await {
            let bytes = serde_json::to_string(&result).unwrap();
            tracing::info!(message_length = bytes.len(), "message={bytes:x?}");

            if let Err(err) = self.write(result).await {
                tracing::warn!(socket_sender_error=%err);
            }
        }

        tracing::info!("closing client writer");
    }

    pub async fn write(&mut self, mesg: client::Message) -> Result<(), axum::Error> {
        let mesg = serde_json::to_string(&mesg).unwrap();
        let mesg = Message::Text(mesg);
        self.sink.send(mesg).await
    }
}
