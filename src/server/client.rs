use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt,
};
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

    pub fn create_handles<'a>(self) -> (ReaderHalf<'a>, WriterHalf) {
        let (tx, rx) = mpsc::unbounded_channel();

        let rhalf = ReaderHalf::new(self.id, self.reader, tx);
        let whalf = WriterHalf::new(self.id, self.writer, rx);

        (rhalf, whalf)
    }

    pub async fn send(&mut self, mesg: Message) -> Result<(), axum::Error> {
        self.writer.send(mesg).await
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}
