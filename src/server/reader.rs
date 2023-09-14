use axum::extract::ws::{self, WebSocket};
use futures::{stream::SplitStream, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::{
    model::client::{self, Message, MessageBody},
    MailBox, Receiver,
};

#[cfg_attr(debug_assertions, allow(dead_code))]
pub struct ReaderHalf {
    id: Uuid,
    stream: SplitStream<WebSocket>,
    mailbox: MailBox<Receiver>,
    tx: mpsc::UnboundedSender<Message>,
}

impl ReaderHalf {
    pub fn new(
        id: Uuid,
        stream: SplitStream<WebSocket>,
        tx: mpsc::UnboundedSender<Message>,
    ) -> Self {
        Self {
            mailbox: MailBox::instance(),

            stream,
            id,
            tx,
        }
    }
}

impl ReaderHalf {
    #[tracing::instrument(skip_all, fields(id = self.id.to_string()))]
    pub async fn spawn_handler(mut self) {
        loop {
            use ws::Message as M;
            tokio::select! {
            Some(Ok(ws_mesg)) = self.stream.next() => match ws_mesg {
                M::Text(string) => {
                    let message = serde_json::from_str(&string).unwrap_or_else(|err| Message::server_message(MessageBody::Response {
                        target: self.id,
                        body: err.to_string().into()
                    }));

                    let _ = self.send(message);
                },
                M::Binary(bytes) => {
                    let message = serde_json::from_slice(&bytes).unwrap_or_else(|err| Message::server_message(MessageBody::Response {
                        target: self.id,
                        body: err.to_string().into()
                    }));

                    let _ = self.send(message);
                },
                _mesg => {}
            },
            Some(_mesg) = self.mailbox.recv(self.id) => {
                let _ = self.send(_mesg);
            },
            }
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn send(&mut self, mesg: Message) -> Result<(), mpsc::error::SendError<client::Message>> {
        self.tx.send(mesg)
    }
}
