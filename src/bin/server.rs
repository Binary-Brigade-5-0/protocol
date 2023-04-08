use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use uuid::Uuid;

use etron::handler::stream_handler;
use etron::Registry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind(SocketAddr::from(([0u8; 4], 3000))).await?;
    // let room_register: Register = Arc::new(Mutex::new(HashMap::new()));
    let register = Arc::new(Mutex::new(Registry::default()));

    println!("Server started at 0.0.0.0:3000...");

    loop {
        let register = Arc::clone(&register);
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
