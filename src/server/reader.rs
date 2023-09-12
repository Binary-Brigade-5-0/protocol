use std::task::Poll;

use axum::extract::ws::{self, WebSocket};
use futures::{future::BoxFuture, stream::SplitStream, FutureExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::{
    model::client::{Message, MessageBody, MessageBuilder},
    MailBox, Receiver,
};

#[cfg_attr(debug_assertions, allow(dead_code))]
pub struct ReaderHalf<'a> {
    id: Uuid,
    stream: SplitStream<WebSocket>,
    mailbox: MailBox<Receiver>,
    tx: mpsc::UnboundedSender<Message>,

    task_list: Vec<BoxFuture<'a, anyhow::Result<()>>>,
}

impl ReaderHalf<'_> {
    pub fn new(
        id: Uuid,
        stream: SplitStream<WebSocket>,
        tx: mpsc::UnboundedSender<Message>,
    ) -> Self {
        Self {
            mailbox: MailBox::instance(),
            task_list: vec![],

            stream,
            id,
            tx,
        }
    }
}

impl ReaderHalf<'_> {
    #[tracing::instrument(skip_all)]
    async fn read_blob(&mut self) -> anyhow::Result<()> {
        use ws::Message as M;
        tokio::select! {
        Some(Ok(ws_mesg)) = self.stream.next()  => match ws_mesg {
            M::Text(string) => {
                let message = serde_json::from_str(&string)?;
                let _ = self.tx.send(message);
            },
            M::Binary(bytes) => {
                let message = serde_json::from_slice(&bytes)?;
                let _ = self.tx.send(message);
            },
            _mesg => {}
        },
        Some(_mesg) = self.mailbox.recv(self.id) => {},
        }

        Ok(())
    }

    pub async fn spawn_handler(mut self) {
        loop {
            self.read_blob().await;
        }
    }
}
