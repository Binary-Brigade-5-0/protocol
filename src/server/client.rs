use axum::extract::ws::{Message, WebSocket};
use futures::stream::{SplitSink, SplitStream};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::{reader::ReaderHalf, writer::WriterHalf};

pub struct Client {
    id: Uuid,
    reader: SplitStream<WebSocket>,
    writer: SplitSink<WebSocket, Message>,
}

impl Client {
    pub fn new(reader: SplitStream<WebSocket>, writer: SplitSink<WebSocket, Message>) -> Self {
        let id = Uuid::new_v4();
        Self { id, reader, writer }
    }

    pub fn create_handles(self) -> (ReaderHalf, WriterHalf) {
        let (tx, rx) = mpsc::unbounded_channel();

        let rhalf = ReaderHalf::new(self.id, self.reader, tx);
        let whalf = WriterHalf::new(self.id, self.writer, rx);

        (rhalf, whalf)
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}
