#![cfg_attr(debug_assertions, allow(dead_code))]
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, OnceLock},
};

use dashmap::DashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::model::client;

static RAW_MAILBOX: OnceLock<Arc<DuplexMailbox>> = OnceLock::new();

#[derive(Default)]
pub struct DuplexMailbox(DashMap<Uuid, mpsc::UnboundedSender<client::Message>>);
pub struct MailboxReader(Uuid, mpsc::UnboundedReceiver<client::Message>);
pub struct MailboxWriter(Arc<DuplexMailbox>);

impl DuplexMailbox {
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

    pub fn add_client(self: Arc<Self>, client_id: Uuid) -> Option<MailboxReader> {
        (!self.0.contains_key(&client_id)).then(|| {
            let (tx, rx) = mpsc::unbounded_channel();
            self.0.insert(client_id, tx);

            MailboxReader(client_id, rx)
        })
    }

    pub fn mailbox_writer() -> MailboxWriter {
        MailboxWriter(Self::instance())
    }
}

impl MailboxWriter {
    pub fn send(&self, key: Uuid, mesg: client::Message) -> anyhow::Result<()> {
        let Self(mailbox) = self;
        let sender = mailbox
            .0
            .get(&key)
            .ok_or_else(|| anyhow::format_err!("Client does not exist"))?;

        Ok(sender.send(mesg)?)
    }
}

impl MailboxReader {
    pub fn hash_id(&self) -> Uuid {
        self.0
    }
}

impl Deref for MailboxReader {
    type Target = mpsc::UnboundedReceiver<client::Message>;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}

impl DerefMut for MailboxReader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.1
    }
}

impl Drop for MailboxReader {
    fn drop(&mut self) {
        DuplexMailbox::instance().0.remove(&self.hash_id());
        self.1.close();
    }
}
