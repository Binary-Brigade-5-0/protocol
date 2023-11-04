use std::{
    sync::{Arc, OnceLock},
    time::Duration,
};

use dashmap::DashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::model::client;

static RAW_MAILBOX: OnceLock<Arc<DrainedMailbox>> = OnceLock::new();

#[derive(Default)]
pub struct DrainedMailbox(DashMap<Uuid, mpsc::UnboundedSender<client::Message>>);
pub struct DrainedReader(Uuid, mpsc::UnboundedReceiver<client::Message>);

impl DrainedMailbox {
    fn new() -> Self {
        if cfg!(debug_assertions) {
            Self::default()
        } else {
            let capacity = crate::settings::Settings::instance().dmap_capacity();
            tracing::debug!("pre-allocating for {capacity} entries to prevent global locking");
            Self::with_capacity(capacity)
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        let senders = DashMap::with_capacity(capacity);
        Self(senders)
    }

    pub fn instance() -> Arc<Self> {
        let mailbox = RAW_MAILBOX.get_or_init(|| Arc::new(Self::new()));
        Arc::clone(mailbox)
    }

    pub fn add_client(self: Arc<Self>, client_id: Uuid) -> Option<DrainedReader> {
        (!self.0.contains_key(&client_id)).then(|| {
            let (tx, rx) = mpsc::unbounded_channel();
            self.0.insert(client_id, tx);

            DrainedReader(client_id, rx)
        })
    }
}

impl DrainedReader {
    pub async fn drain(mut self, timeout: Duration) -> Box<[client::Message]> {
        let mut buffer = vec![];
        let sleeper = async { std::thread::sleep(timeout) };
        tokio::pin!(sleeper);

        loop {
            tokio::select! {
            _ = &mut sleeper  => break,
            m = self.1.recv() => match m {
                Some(mesg) => buffer.push(mesg),
                None       => break,
            }
            }
        }

        buffer.into_boxed_slice()
    }

    pub fn hash_id(&self) -> Uuid {
        self.0
    }
}

impl Drop for DrainedReader {
    fn drop(&mut self) {
        DrainedMailbox::instance().0.remove(&self.hash_id());
        self.1.close();
    }
}
