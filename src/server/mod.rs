pub(crate) mod client;
pub(crate) mod reader;
pub(crate) mod writer;

use axum::extract::ws::WebSocket;
use futures::StreamExt;
use tokio::sync::{broadcast, mpsc};

use crate::message::{
    model::client::{Message, MessageBody},
    MailBox, Sender,
};

#[cfg_attr(debug_assertions, allow(dead_code))]
pub struct Server {
    mailbox: MailBox<Sender>,
    client_rx: mpsc::UnboundedReceiver<Message>,
    broadcast_tx: broadcast::Sender<Message>,
}

#[cfg_attr(debug_assertions, allow(dead_code))]
impl Server {
    #[rustfmt::skip]
    pub fn new(client_rx: mpsc::UnboundedReceiver<Message>) -> Self {
        let mailbox = MailBox::instance();
        let (broadcast_tx, ..) = broadcast::channel(128);

        Self { mailbox, client_rx, broadcast_tx }
    }

    pub fn broadcast_receiver(&self) -> broadcast::Receiver<Message> {
        self.broadcast_tx.subscribe()
    }

    pub async fn spawn_server(mut self) {
        loop {
            if let Some(mesg) = self.client_rx.recv().await {
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
            }
        }
    }
}

#[cfg_attr(debug_assertions, allow(dead_code))]
impl Server {
    #[tracing::instrument(skip_all)]
    pub(crate) async fn spawn_client(websocket: WebSocket) {
        let (writer, reader) = websocket.split();

        let client = client::Client::new(reader, writer);
        let mesg = Message::builder()
            .body(MessageBody::Response {
                target: client.id(),
                body: "".into(),
            })
            .build();

        let (mut rhalf, whalf) = client.create_handles();
        let _ = rhalf.send(mesg);

        tokio::spawn(rhalf.spawn_reader());
        tokio::spawn(whalf.spawn_writer());
    }
}
