pub mod message;

use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::RoomRegister;

async fn broadcast_message(
    my_id: Uuid,
    message: message::Message,
    room_register: RoomRegister,
) -> Result<(), tokio::sync::mpsc::error::SendError<message::Message>> {
    room_register
        .lock()
        .await
        .iter()
        .filter(|(k, _)| k != &&my_id)
        .try_for_each(|(_, s)| s.send(message.clone()))
}

fn handle_post(content: String) {}
fn handle_ok(content: String) {}
fn handle_err(content: String) {}
fn handle_request(content: String) {}

#[allow(clippy::unit_arg)]
fn handle_message(
    message::Message {
        kind,
        time,
        message,
    }: message::Message,
) -> anyhow::Result<()> {
    use message::Kind;
    match kind {
        Kind::Post => Ok(handle_post(message)),
        Kind::Ok => Ok(handle_ok(message)),
        Kind::Err => Ok(handle_err(message)),
        Kind::Request => Ok(handle_request(message)),

        e => Err(anyhow::anyhow!("invalid client side message: {:?}", e)),
    }
}

pub async fn stream_handler(
    stream: TcpStream,
    addr: SocketAddr,
    sockid: Uuid,
    register: RoomRegister,
) {
    let (reader, mut writer) = stream.into_split();
    let mut breader = BufReader::new(reader);
    let mut buffer = String::new();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let message = message::MessageBuilder::default()
        .message(sockid.to_string())
        .kind(message::Kind::Created)
        .time(SystemTime::now().into())
        .build()
        .unwrap();

    writer
        .write_all(&serde_json::to_vec(&message).unwrap())
        .await;

    {
        let tx = tx.clone();
        register.lock().await.insert(sockid, tx);
    }

    loop {
        tokio::select! {
            result = breader.read_line(&mut buffer) => match result {
                Ok(0)  => {
                    println!("Connection to {}({}), closed...", sockid, addr);
                    break;
                },
                Ok(_)  => {
                    let register = Arc::clone(&register);
                    let msg = match serde_json::from_str::<message::Message>(&buffer) {
                        Err(e) => {
                            tx.send(message::MessageBuilder::default()
                                    .message(format!("invalid message format, reason: {:?}", e))
                                    .time(SystemTime::now().into())
                                    .kind(message::Kind::Err)
                                    .build()
                                    .unwrap());
                            continue;
                        },
                        Ok(m) => m,
                    };

                    if let Err(e) = handle_message(msg) {
                        tx.send(message::MessageBuilder::default()
                            .message(e.to_string())
                            .kind(message::Kind::Err)
                            .time(SystemTime::now().into())
                            .build()
                            .unwrap());
                    }

                    if let Err(e) = broadcast_message(sockid, message::MessageBuilder::default()
                        .message(buffer.clone())
                        .kind(message::Kind::Post)
                        .time(SystemTime::now().into())
                        .build()
                        .unwrap(), register).await {
                            eprintln!("{:?}", e);
                        };

                    buffer.clear();
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                    break;
                }
            },
            Some(another) = rx.recv() => {
                let json = serde_json::to_string(&another).unwrap();
                println!("{}", json);
                writer.write_all(json.as_bytes()).await;
            }
        }
    }
}
