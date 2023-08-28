use tokio::sync::{broadcast, mpsc};

lazy_static::lazy_static! {
    static ref SENDER_MAP: () = ();
}

pub struct Channels {
    pub broadcast_rx: broadcast::Receiver<Box<[u8]>>,
    pub client_tx: mpsc::Sender<Box<[u8]>>,
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
    broadcast_tx: broadcast::Sender<Box<[u8]>>,
    client_rx: mpsc::Receiver<Box<[u8]>>,
    server_rx: mpsc::UnboundedReceiver<Box<[u8]>>,
}

impl MsgService {
    pub fn new(server_rx: mpsc::UnboundedReceiver<Box<[u8]>>) -> (Self, Channels) {
        let (broadcast_tx, broadcast_rx) = broadcast::channel(256);
        let (client_tx, client_rx) = mpsc::channel(1024);

        let this = Self {
            broadcast_tx,
            client_rx,
            server_rx,
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
            Some(message) = self.server_rx.recv() => {
                tracing::info!(message.len=%message.len(), ?message, "broadcasting message from server");
                self.broadcast_tx.send(message);
            },
            }
        }
    }
}
