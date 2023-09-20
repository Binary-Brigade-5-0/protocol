use axum::extract::ws::{self, WebSocket};
use futures::{stream::SplitStream, StreamExt};
use tokio::sync::mpsc;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::message::{
    model::client::{Message, MessageBody},
    MailBox, Receiver,
};

use super::client::Channels;

#[cfg_attr(debug_assertions, allow(dead_code))]
#[derive(TypedBuilder)]
// #[builder(pattern = "owned")]
pub struct ReaderHalf {
    id: Uuid,
    stream: SplitStream<WebSocket>,
    writer_tx: mpsc::UnboundedSender<Message>,
    channels: Channels,

    #[builder(setter(skip), default = crate::message::MailBox::instance())]
    mailbox: MailBox<Receiver>,
}

impl ReaderHalf {
    #[tracing::instrument(skip_all, fields(id = self.id.to_string()))]
    pub async fn spawn_reader(mut self) {
        let message_maker = |maybe_message: serde_json::Result<Message>| -> Message {
            let make_err = |err: serde_json::Error| MessageBody::Error {
                criminal: self.id,
                error: err.to_string().into(),
            };

            maybe_message.unwrap_or_else(|err| Message::builder().body(make_err(err)).build())
        };

        loop {
            use ws::Message as M;
            tokio::select! {
            ws_mesg = self.stream.next() => match ws_mesg {
                Some(Ok(ws_mesg)) => match ws_mesg {
                    M::Text(string) => {
                        let message = message_maker(serde_json::from_str(&string));
                        if let Err(err) = self.channels.send_server(message) {
                            tracing::error!(channel_error=%err)
                        }
                    },
                    M::Binary(bytes) => {
                        let message = message_maker(serde_json::from_slice(&bytes));
                        if let Err(err) = self.channels.send_server(message) {
                            tracing::error!(channel_error=%err)
                        }
                    },

                    _mesg => {},
                },
                Some(Err(ws_err)) => { tracing::error!(websocket_error=%ws_err); break },
                None => break,
            },

            Some(mesg) = self.mailbox.recv(self.id) => if let Err(err) = self.writer_tx.send(mesg) {
                tracing::error!(writer_tx_error=%err);
            },

            broadcast = self.channels.get_broadcast() => match broadcast {
                Err(err) => tracing::warn!(broadcast_receive_error=%err),

                Ok(mesg) if *mesg.sender() == self.id => tracing::debug!("skipping broadcast from self"),
                Ok(mesg) => if let Err(err) = self.writer_tx.send(mesg) {
                    tracing::error!(writer_tx_error=%err);
                }
            },
            }
        }

        tracing::info!("closing client reader");
    }
}
