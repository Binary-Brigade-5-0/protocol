use tokio::sync::{broadcast, mpsc};

use crate::message::{MailBox, Message, Sender, ServerMsg};

lazy_static::lazy_static! {
    static ref SENDER_MAP: () = ();
}

pub struct Channels {
    pub broadcast_rx: broadcast::Receiver<Message>,
    pub client_tx: mpsc::Sender<Message>,
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
    broadcast_tx: broadcast::Sender<Message>,
    client_rx: mpsc::Receiver<Message>,
    server_rx: mpsc::UnboundedReceiver<ServerMsg>,

    mailbox: MailBox<Sender>,
}

impl MsgService {
    pub fn new(server_rx: mpsc::UnboundedReceiver<ServerMsg>) -> (Self, Channels) {
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
            Some(message) = self.client_rx.recv() => tracing::trace!(message.len=%message.len(), ?message),
            // Some(message) = self.server_rx.recv() => {
            //     tracing::info!(message.len=%message.len(), ?message, "broadcasting message from server");
            //     self.broadcast_tx.send(message);
            // },
            Some(message) = self.server_rx.recv() => match message {
                ServerMsg::Connected(id)    => {
                    self.mailbox.add_client(id);
                    self.mailbox.send(id, format!("{id}\r\n").as_bytes().into()).await;
                },
                ServerMsg::Disconnected(id) => {
                    self.mailbox.remove_client(id).await;
                },

                ServerMsg::Broadcast(bytes) => {
                    self.broadcast_tx.send(bytes);
                }
            }
            }
        }
    }
}
