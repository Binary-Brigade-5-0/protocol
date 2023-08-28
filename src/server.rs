use dashmap::DashMap;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, task::Poll};
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::{
    broker::{Channels, MsgService},
    client::ClientHandle,
    message::ServerMsg,
};

#[allow(dead_code)]
type MessangerRecord = Arc<DashMap<Uuid, mpsc::UnboundedReceiver<Box<[u8]>>>>;

pub struct Server {
    listener: TcpListener,
    client_handles: HashMap<Uuid, JoinHandle<std::io::Result<()>>>,
    channels: Channels,
    msg_tx: mpsc::UnboundedSender<ServerMsg>,
}

impl Server {
    pub async fn new(addr: SocketAddr) -> std::io::Result<Self> {
        tracing::info!(server_addr=%addr, "starting server...");
        let listener = TcpListener::bind(addr).await?;
        let (tx, rx) = mpsc::unbounded_channel();

        let (msgserv, channels) = MsgService::new(rx);
        tokio::spawn(msgserv.handler());

        Ok(Self {
            listener,
            client_handles: Default::default(),
            channels,
            msg_tx: tx,
        })
    }
}

impl std::future::Future for Server {
    type Output = std::io::Result<()>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match self.listener.poll_accept(cx) {
            Poll::Ready(Ok((stream, addr))) => {
                let client = ClientHandle::new(stream, self.channels.clone());
                let client_id = client.id();

                tracing::trace!(%client_id, client_addr=%addr);
                tracing::info!("client {client_id} has connected");

                self.msg_tx.send(ServerMsg::Connected(client_id));

                let task = tokio::spawn(client.handler());
                self.get_mut().client_handles.insert(client_id, task);

                cx.waker().wake_by_ref();
            }
            Poll::Ready(Err(e)) => tracing::error!(server_stream_error=%e),
            Poll::Pending => {
                let _self = self.get_mut();
                for (client, handle) in _self
                    .client_handles
                    .iter_mut()
                    .filter(|(_, handle)| handle.is_finished())
                {
                    tracing::debug!(%client, "removing from task handles list");
                    _self.msg_tx.send(ServerMsg::Disconnected(*client));
                    tokio::pin!(handle);
                    let Poll::Ready(Err(err)) = handle.poll(cx) else {
                        continue;
                    };

                    tracing::warn!(client_handle_error=%err);
                }

                _self
                    .client_handles
                    .retain(|_, handle| !handle.is_finished());
            }
        }

        Poll::Pending
    }
}
