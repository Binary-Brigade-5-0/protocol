#![allow(unused)]
pub mod handler;

mod key {
    use lazy_static::lazy_static;
    use openssl::{pkey::Private, rsa::Rsa};

    #[rustfmt::skip]
    lazy_static! {
        pub static ref K_E2E: Rsa<Private>     = Rsa::generate(2048).unwrap();
        pub static ref K_STORAGE: Rsa<Private> = Rsa::generate(2048).unwrap();
    }
}

use handler::message::Message;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};
use uuid::Uuid;

pub type Rx = UnboundedReceiver<Message>;
pub type Tx = UnboundedSender<Message>;

type ClientRegister = HashMap<Uuid, (Tx, Rx)>;
type PostsRegister = HashMap<Uuid, HashSet<Uuid>>;

#[derive(Default)]
pub struct Registry {
    clients: ClientRegister,
    posts: PostsRegister,
}

impl Registry {
    pub fn clients(&self) -> &ClientRegister {
        &self.clients
    }

    pub fn posts(&self) -> &PostsRegister {
        &self.posts
    }
}

impl Registry {
    pub fn add_client(&mut self, cid: Uuid, tx: Tx, rx: Rx) {
        self.clients.insert(cid, (tx, rx));
    }

    pub fn remove_client(&mut self, client_id: Uuid) {
        self.clients.remove(&client_id);

        self.posts
            .iter_mut()
            .filter(|(_, v)| v.contains(&client_id))
            .for_each(|(k, v)| {
                v.remove(&client_id);
            });
    }
}

impl Registry {
    pub fn add_post(&mut self, post_id: Uuid, client_id: Uuid) {
        self.posts
            .entry(post_id)
            .or_insert_with(HashSet::new)
            .insert(client_id);
    }

    pub fn remove_post(&mut self, post_id: Uuid) {
        self.posts.remove(&post_id);
    }
}

type Register = Arc<Mutex<Registry>>;
