use axum::extract::ws::{self, WebSocket};
use derive_builder::Builder;
use futures::{stream::SplitStream, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::{
    model::client::{Message, MessageBody},
    MailBox, Receiver,
};

use super::client::Channels;

#[cfg_attr(debug_assertions, allow(dead_code))]
#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct ReaderHalf {
    id: Uuid,
    stream: SplitStream<WebSocket>,
    writer_tx: mpsc::UnboundedSender<Message>,
    channels: Channels,

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

                        let _ = self.channels.send_server(message);
                    },
                    M::Binary(bytes) => {
                        let message = serde_json::from_slice(&bytes);
                        let message = message.unwrap_or_else(|err| Message::builder().body(MessageBody::Response {
                            target: self.id,
                            body: err.to_string().into()
                        }).build());

                        let _ = self.writer_tx.send(message);
                    },

                    _mesg => {},
                },
                Some(Err(ws_err)) => { tracing::error!(websocket_error=%ws_err); break },
                None => break,
            },
            Some(_mesg) = self.mailbox.recv(self.id) => {
                let _ = self.writer_tx.send(_mesg);
            },
            broadcast = self.channels.get_broadcast() => match broadcast {
                Ok(mesg) if *mesg.sender() == self.id => tracing::debug!("skipping broadcast from self"),
                Err(err) => tracing::warn!(broadcast_receive_error=%err),
                Ok(mesg) => {
                    let _ = self.writer_tx.send(mesg);
                }
            },
            }
        }

        tracing::info!("closing client reader");
    }
}
