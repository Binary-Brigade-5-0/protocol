use axum::extract::ws::{Message, WebSocket};
use futures::{stream::SplitSink, SinkExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::model::client;

#[cfg_attr(debug_assertions, allow(dead_code))]
pub struct WriterHalf {
    id: Uuid,
    sink: SplitSink<WebSocket, Message>,
    rx: mpsc::UnboundedReceiver<client::Message>,
}

impl WriterHalf {
    pub fn new(
        id: Uuid,
        sink: SplitSink<WebSocket, Message>,
        rx: mpsc::UnboundedReceiver<client::Message>,
    ) -> Self {
        Self { id, sink, rx }
    }
}

impl WriterHalf {
    pub async fn spawn_handler(mut self) -> anyhow::Result<()> {
        loop {
            if let Some(result) = self.rx.recv().await {
                let bytes = serde_json::to_string(&result).unwrap();
                self.sink.send(Message::Text(bytes.clone())).await?;
                tracing::info!(message_length = bytes.len(), "message={bytes:x?}");
            };
        }
    }
}
