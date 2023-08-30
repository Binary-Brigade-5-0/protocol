use crate::{
    broker::Channels,
    message::{model::client::Message, MailBox, Receiver},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use uuid::Uuid;

#[rustfmt::skip]
#[derive(thiserror::Error)]
#[derive(Debug)]
pub enum MessageError {
    #[error("client disconnected")]
    Disconnected,

    #[error("{0}")]
    IOError(#[from] std::io::Error),

    #[error("{0}")]
    DeserializeError(#[from] serde_json::Error)
}

pub struct ClientHandle {
    channels: Channels,
    id: Uuid,

    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,

    mailbox: MailBox<Receiver>,
}

impl ClientHandle {
    pub fn new(stream: TcpStream, channels: Channels) -> Self {
        let (reader, writer) = stream.into_split();
        let (reader, writer) = (BufReader::new(reader), BufWriter::new(writer));

        Self {
            channels,
            id: Uuid::new_v4(),
            reader,
            writer,

            mailbox: MailBox::instance(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

impl ClientHandle {
    #[tracing::instrument(skip_all, fields(id = self.id.to_string()))]
    pub async fn handler(mut self) -> std::io::Result<()> {
        tracing::info!("spawning client task...");
        let mut rdbuf = Vec::with_capacity(1024);

        loop {
            tokio::select! {
            m_result = ClientHandle::read_msg(&mut self.reader, &mut rdbuf) => match m_result {
                Err(MessageError::Disconnected) => break,
                Err(MessageError::IOError(err)) => tracing::error!("{err}"),
                Err(MessageError::DeserializeError(err)) => {
                    tracing::warn!("{err}");
                    self.writer.write_all(format!("{err}").as_bytes()).await;
                    self.writer.flush().await;
                },

                Ok(message) => {
                    self.channels.client_tx.send(message).await;
                    rdbuf.clear();
                }
            },
            broadcast = self.channels.broadcast_rx.recv() => match broadcast {
                Ok(broadcast) => {
                    let bytes = serde_json::to_vec(&broadcast).unwrap();
                    self.writer.write_all(&bytes).await;
                    self.writer.flush().await;
                },
                Err(e) => tracing::warn!(broadcast_recv_error=%e),
            },
            Some(message) = self.mailbox.recv(self.id) => {
                let bytes = serde_json::to_vec(&message).unwrap();
                self.writer.write_all(&bytes).await;
                self.writer.flush().await;
            },
            }
        }

        tracing::info!("destroying client task");
        Ok(())
    }

    async fn read_msg(
        reader: &mut BufReader<OwnedReadHalf>,
        buffer: &mut Vec<u8>,
    ) -> Result<Message, MessageError> {
        match reader.read_until(b'\n', buffer).await {
            Ok(0) => Err(MessageError::Disconnected),
            Err(e) => Err(MessageError::IOError(e)),

            Ok(len) => {
                tracing::trace!(bytes=?buffer, length=%len);
                let message = serde_json::from_slice(buffer)?;
                Ok(message)
            }
        }
    }
}
