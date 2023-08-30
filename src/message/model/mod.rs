pub mod client;

pub mod server {
    use uuid::Uuid;

    pub enum Message {
        Connected(Uuid),
        Disconnected(Uuid),
        Broadcast(super::client::Message),
    }
}
