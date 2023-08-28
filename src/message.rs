// TODO: formulate a proper way to create a singleton for the mailbox component

use dashmap::DashMap;
use tokio::sync::mpsc::{self, error::SendError};
use uuid::Uuid;

// #[derive(Clone, Copy)]
// pub enum Sender {
//     Admin,
//     Client(Uuid),
// }

// #[derive(Builder)]
// pub struct Message {
//     sender: Sender,
//     body: Box<[u8]>,
// }

pub type Message = Box<[u8]>;
pub trait ReaderType {}

pub struct Receiver;
pub struct Sender;

impl ReaderType for Receiver {}
impl ReaderType for Sender {}

#[rustfmt::skip]
#[derive(Debug)]
#[derive(thiserror::Error)]
pub enum Error {
    #[error("client {0} does not exist")]
    DoesNotExist(Uuid),

    #[error("failed sending message: {0}")]
    SenderError(#[from] SendError<Message>),
}

pub struct MailBox<T: ReaderType> {
    rx_map: DashMap<Uuid, mpsc::UnboundedReceiver<Message>>,
    tx_map: DashMap<Uuid, mpsc::UnboundedSender<Message>>,

    _type: std::marker::PhantomData<T>,
}

impl<T: ReaderType> MailBox<T> {
    fn new() -> Self {
        Self {
            rx_map: DashMap::new(),
            tx_map: DashMap::new(),
            _type: Default::default(),
        }
    }

    pub fn add_client(&self, client_id: Uuid) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.rx_map.insert(client_id, rx);
        self.tx_map.insert(client_id, tx);
    }
}

impl MailBox<Sender> {
    pub async fn recv(&self, client_id: Uuid) -> Option<Message> {
        self.rx_map.get_mut(&client_id)?.recv().await
    }
}

impl MailBox<Receiver> {
    pub async fn send(&self, client_id: Uuid, message: Message) -> Result<(), Error> {
        self.tx_map
            .get_mut(&client_id)
            .ok_or(Error::DoesNotExist(client_id))?
            .send(message)?;

        Ok(())
    }
}

impl<T: ReaderType> Default for MailBox<T> {
    fn default() -> Self {
        Self::new()
    }
}
