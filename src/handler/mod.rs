pub mod message;

use crate::Register;
use message::Message;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use uuid::Uuid;

async fn broadcast_message(
    my_id: Uuid,
    message: message::Message,
    room_register: Register,
) -> Result<(), tokio::sync::mpsc::error::SendError<message::Message>> {
    room_register
        .lock()
        .await
        .clients()
        .iter()
        .filter(|(k, _)| k != &&my_id)
        .try_for_each(|(_, s)| s.0.send(message.clone()))
}

async fn handle_server_post(content: String, register: Register, sockid: Uuid) -> Message {
    let postid = Uuid::new_v4();
    register.lock().await.add_post(postid, sockid);

    Message {
        kind: message::Kind::Response,
        time: SystemTime::now().into(),
        message: format!("{}:{}", postid, content),
    }
}

async fn handle_ok(content: String, register: Register) -> Message {
    todo!()
}

async fn handle_err(content: String, register: Register) -> Message {
    todo!()
}

async fn handle_server_request(postid: Uuid, register: Register) -> Message {
    if postid == Uuid::nil() {
        let mut mvec = vec![];
        let mut reglock = register.lock().await;

        reglock
            .posts()
            .iter()
            .for_each(|(pid, set)| set.iter().for_each(|sid| mvec.push((*pid, *sid))));

        let clients = &mut reglock.clients;

        for (pid, sid) in mvec.iter_mut() {
            let mut client = clients.get_mut(sid).unwrap();
            client.0.send(Message {
                kind: message::Kind::Request,
                time: SystemTime::now().into(),
                message: pid.to_string(),
            });

            let response = client.1.recv().await;
            println!("{:?}", response);
        }
    }

    Message {
        kind: message::Kind::Ok,
        time: SystemTime::now().into(),
        message: "asd".to_string(),
    }
}

async fn handle_client_response(content: String) -> Message {
    todo!()
}

#[allow(clippy::unit_arg)]
async fn handle_message(
    Message {
        kind,
        time,
        message,
    }: Message,
    register: Register,
    sockid: Uuid,
) -> anyhow::Result<Message> {
    use message::Kind;

    // if SystemTime::now().duration_since(time.into()).is_err() {
    //     return Err(anyhow::anyhow!("invalid message timestamp on message"));
    // }

    match kind {
        Kind::Post => Ok(handle_server_post(message, register, sockid).await),
        Kind::Ok => Ok(handle_ok(message, register).await),
        Kind::Err => Ok(handle_err(message, register).await),
        Kind::Request => Ok(handle_server_request(Uuid::from_str(&message)?, register).await),
        Kind::Response => Ok(handle_client_response(message).await),

        e => Err(anyhow::anyhow!("invalid client side message: {:?}", e)),
    }
}

pub async fn stream_handler(stream: TcpStream, addr: SocketAddr, sockid: Uuid, register: Register) {
    let (reader, mut writer) = stream.into_split();

    let mut breader = BufReader::new(reader);
    let mut buffer = String::new();

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let (tx_inner, rx_inner) = tokio::sync::mpsc::unbounded_channel();

    let message = Message {
        message: sockid.to_string(),
        time: SystemTime::now().into(),
        kind: message::Kind::Created,
    };

    writer
        .write_all(&serde_json::to_vec(&message).unwrap())
        .await;

    {
        register
            .lock()
            .await
            .add_client(sockid, tx.clone(), rx_inner);
    }

    loop {
        let register = Arc::clone(&register);
        tokio::select! {
            result = breader.read_line(&mut buffer) => match result {
                Ok(0) => break,
                Ok(_) => {
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

                    if let message::Kind::Response = &msg.kind {
                        tx_inner.send(msg);
                    } else {
                        let _register = Arc::clone(&register);
                        match handle_message(msg, _register, sockid).await {
                            Err(e) => tx.send(Message {
                                kind: message::Kind::Err,
                                time: SystemTime::now().into(),
                                message: e.to_string()
                            }),
                            Ok(m) => tx.send(m),
                        };
                    }

                    if let Err(e) = broadcast_message(sockid, Message {
                        message: buffer.clone(),
                        kind: message::Kind::Post,
                        time: SystemTime::now().into(),
                    }, Arc::clone(&register)).await
                    {
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

    register.lock().await.remove_client(sockid);
    println!("Connection to {}({}), closed...", sockid, addr);
}
