use axum::extract::ws::{Message, WebSocket};
use futures::stream::{SplitSink, SplitStream};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::{
    reader::{ReaderHalf, ReaderHalfBuilder},
    writer::{WriterHalf, WriterHalfBuilder},
};

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

        let rhalf = ReaderHalfBuilder::default()
            .id(self.id)
            .stream(self.reader)
            .tx(tx);

        let whalf = WriterHalfBuilder::default()
            .id(self.id)
            .sink(self.writer)
            .rx(rx);

        let rhalf = rhalf.build().unwrap();
        let whalf = whalf.build().unwrap();

        (rhalf, whalf)
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}
