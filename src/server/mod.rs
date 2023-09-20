pub(crate) mod client;
pub(crate) mod reader;
pub(crate) mod writer;

use std::sync::Arc;

use axum::extract::ws::WebSocket;
use futures::StreamExt;
use tokio::sync::{broadcast, mpsc};

use crate::message::{
    model::client::{Message, MessageBody},
    MailBox, Sender,
};

use client::Channels;

#[cfg_attr(debug_assertions, allow(dead_code))]
pub struct Server {
    client_rx: mpsc::UnboundedReceiver<Message>,
    broadcast_tx: broadcast::Sender<Message>,
    channels: Option<Channels>,

    mailbox: MailBox<Sender>,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
pub struct TaskSpawner {
    channels: Channels,
    mailbox: MailBox<Sender>,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
impl Server {
    pub fn new() -> Self {
        let mailbox = MailBox::instance();
        let (client_tx, client_rx) = mpsc::unbounded_channel();
        let (broadcast_tx, broadcast_rx) = broadcast::channel(128);

        let channels = Channels::builder()
            .server_tx(client_tx)
            .broadcast_rx(broadcast_rx)
            .build();

        Self {
            mailbox,
            client_rx,
            broadcast_tx,
            channels: Some(channels),
        }
    }

    pub fn task_spawner(&mut self) -> Option<Arc<TaskSpawner>> {
        let channels = self.channels.take()?;
        let mailbox = MailBox::instance();
        let task_spawner = TaskSpawner { channels, mailbox };
        let task_spawner = Arc::new(task_spawner);

        Some(task_spawner)
    }

    pub fn broadcast_receiver(&self) -> broadcast::Receiver<Message> {
        self.broadcast_tx.subscribe()
    }

    pub async fn spawn_server(mut self) {
        loop {
            tokio::select! {
            Some(mesg) = self.client_rx.recv()  => {
                tracing::info!(?mesg);
                match mesg.body() {
                    #[rustfmt::skip]
                    MessageBody::Query(..) => if let Err(e) = self.broadcast_tx.send(mesg) {
                        tracing::error!(broadcast_sender_err=%e);
                    },

                    #[rustfmt::skip]
                    MessageBody::Response { target, .. } => if let Err(e) = self.mailbox.send(*target, mesg).await {
                        tracing::error!(mailbox_sender_error=%e);
                    },

                    MessageBody::Get(_uuid) => {},
                    MessageBody::Exists(_uuid) => {},
                }
            },
            }
        }
    }
}

impl TaskSpawner {
    #[tracing::instrument(skip_all)]
    pub async fn spawn_client(self: Arc<Self>, ws: WebSocket) {
        let channels = self.channels.clone();
        let client = Client::new(ws.split());

        tracing::info!("client {} connected", client.id());
        self.mailbox.add_client(client.id());

        let mesg = Message::builder()
            .body(MessageBody::Connected(client.id()))
            .build();

        let client_id = client.id();
        let (rhalf, mut whalf) = client.create_handles(channels);

        if let Err(sink_error) = whalf.write(mesg).await {
            tracing::warn!(%sink_error);
        }

        let reader = tokio::spawn(rhalf.spawn_reader());
        let writer = tokio::spawn(whalf.spawn_writer());

        let (r1, r2) = tokio::join!(reader, writer);

        if let Err(err) = r1.and(r2) {
            tracing::error!(task_join_error=%err);
        }

        tracing::info!("client {client_id} disconnected");
    }
}
