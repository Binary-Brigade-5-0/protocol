use axum::extract::ws::{Message, WebSocket};
use futures::stream::{SplitSink, SplitStream};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use crate::message::model::client;

use super::{reader::ReaderHalf, writer::WriterHalf};

pub struct Client {
    id: Uuid,
    reader: SplitStream<WebSocket>,
    writer: SplitSink<WebSocket, Message>,
}

#[derive(typed_builder::TypedBuilder)]
pub struct Channels {
    server_tx: mpsc::UnboundedSender<client::Message>,
    broadcast_rx: broadcast::Receiver<client::Message>,
}

impl Clone for Channels {
    #[rustfmt::skip]
    fn clone(&self) -> Self {
        let broadcast_rx = self.broadcast_rx.resubscribe();
        let server_tx = self.server_tx.clone();

        Self { server_tx, broadcast_rx }
    }
}

impl Channels {
    pub async fn get_broadcast(&mut self) -> Result<client::Message, broadcast::error::RecvError> {
        self.broadcast_rx.recv().await
    }

    pub fn send_server(
        &mut self,
        mesg: client::Message,
    ) -> Result<(), mpsc::error::SendError<client::Message>> {
        self.server_tx.send(mesg)
    }
}

impl Client {
    pub fn new((writer, reader): (SplitSink<WebSocket, Message>, SplitStream<WebSocket>)) -> Self {
        let id = Uuid::new_v4();
        Self { id, reader, writer }
    }

    pub fn create_handles(self, channels: Channels) -> (ReaderHalf, WriterHalf) {
        let (tx, rx) = mpsc::unbounded_channel();

        let rhalf = ReaderHalf::builder()
            .id(self.id)
            .stream(self.reader)
            .channels(channels)
            .writer_tx(tx);

        let whalf = WriterHalf::builder().id(self.id).sink(self.writer).rx(rx);

        (rhalf.build(), whalf.build())
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}
