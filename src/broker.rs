use std::time::SystemTime;

use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

use crate::message::{
    model::client::{self, MessageBuilder},
    model::server,
    MailBox, Sender,
};

pub struct Channels {
    pub broadcast_rx: broadcast::Receiver<client::Message>,
    pub client_tx: mpsc::Sender<client::Message>,
}

impl Clone for Channels {
    fn clone(&self) -> Self {
        Self {
            broadcast_rx: self.broadcast_rx.resubscribe(),
            client_tx: self.client_tx.clone(),
        }
    }
}

pub struct MsgService {
    broadcast_tx: broadcast::Sender<client::Message>,
    client_rx: mpsc::Receiver<client::Message>,
    server_rx: mpsc::UnboundedReceiver<server::Message>,

    mailbox: MailBox<Sender>,
}

impl MsgService {
    pub fn new(server_rx: mpsc::UnboundedReceiver<server::Message>) -> (Self, Channels) {
        let (broadcast_tx, broadcast_rx) = broadcast::channel(256);
        let (client_tx, client_rx) = mpsc::channel(1024);

        let this = Self {
            broadcast_tx,
            client_rx,
            server_rx,

            mailbox: MailBox::instance(),
        };

        let channels = Channels {
            client_tx,
            broadcast_rx,
        };

        (this, channels)
    }

    #[tracing::instrument(name = "message_service", skip_all)]
    pub async fn handler(mut self) -> anyhow::Result<()> {
        loop {
            tokio::select! {
            Some(message) = self.client_rx.recv() => {
                tracing::trace!(message.len=%message.body_length(), ?message);
                match message.body() {
                    client::MessageBody::Query(..) => { self.broadcast_tx.send(message); },
                    client::MessageBody::Response{target, ..} => {
                        self.mailbox.send(*target, message).await;
                    },
                    client::MessageBody::Get(uuid) => {},
                    client::MessageBody::Exists(uuid) => {},
                }
            },
            Some(message) = self.server_rx.recv() => match message {
                server::Message::Connected(id)    => {
                    self.mailbox.add_client(id);
                    let message = MessageBuilder::default()
                        .body(client::MessageBody::Response{ body: id.into_bytes().into(), target: id })
                        .time(SystemTime::now())
                        .sender(Uuid::nil())
                        .build()
                        .unwrap();

                    self.mailbox.send(id, message).await;
                },
                server::Message::Disconnected(id) => {
                    self.mailbox.remove_client(id).await;
                },

                server::Message::Broadcast(msg) => {
                    self.broadcast_tx.send(msg);
                }
            },
            }
        }
    }
}
