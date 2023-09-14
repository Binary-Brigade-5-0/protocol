use axum::extract::ws::{self, WebSocket};
use derive_builder::Builder;
use futures::{stream::SplitStream, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::{
    model::client::{self, Message, MessageBody},
    MailBox, Receiver,
};

#[cfg_attr(debug_assertions, allow(dead_code))]
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct ReaderHalf {
    id: Uuid,
    stream: SplitStream<WebSocket>,
    tx: mpsc::UnboundedSender<Message>,

    #[builder(setter(skip), default = "crate::message::MailBox::instance()")]
    mailbox: MailBox<Receiver>,
}

impl ReaderHalf {
    #[tracing::instrument(skip_all, fields(id = self.id.to_string()))]
    pub async fn spawn_reader(mut self) {
        loop {
            use ws::Message as M;
            tokio::select! {
            ws_mesg = self.stream.next() => match ws_mesg {
                Some(Ok(ws_mesg)) => match ws_mesg {
                    M::Text(string) => {
                        let message = serde_json::from_str(&string);
                        let message = message.unwrap_or_else(|err| Message::builder().body(MessageBody::Response {
                            target: self.id,
                            body: err.to_string().into()
                        }).build());

                        let _ = self.send(message);
                    },
                    M::Binary(bytes) => {
                        let message = serde_json::from_slice(&bytes);
                        let message = message.unwrap_or_else(|err| Message::builder().body(MessageBody::Response {
                            target: self.id,
                            body: err.to_string().into()
                        }).build());

                        let _ = self.send(message);
                    },

                    _mesg => {},
                },
                Some(Err(ws_err)) => { tracing::error!(websocket_error=%ws_err); break },
                None => break,
            },
            Some(_mesg) = self.mailbox.recv(self.id) => {
                let _ = self.send(_mesg);
            },
            }
        }

        tracing::info!("disconnecting from client");
    }

    #[tracing::instrument(skip(self))]
    pub fn send(&mut self, mesg: Message) -> Result<(), mpsc::error::SendError<client::Message>> {
        self.tx.send(mesg)
    }
}
