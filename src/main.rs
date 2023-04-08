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
use handler::stream_handler;
use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::{TcpListener, TcpStream},
};
use uuid::Uuid;

type Rx = UnboundedReceiver<Message>;
type Tx = UnboundedSender<Message>;
type RoomRegister = Arc<Mutex<HashMap<Uuid, Tx>>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from(([0u8; 4], 3000))).await?;
    let room_register: RoomRegister = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let register = Arc::clone(&room_register);
        let Ok((stream, addr)) = listener.accept().await.map_err(|e| dbg!(e)) else {
            continue;
        };

        let sock_id = Uuid::new_v4();

        println!(
            "Connection to socket: {}, with addr: {} established...",
            sock_id, addr
        );

        tokio::spawn(stream_handler(stream, addr, sock_id, register));
    }
}
