use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod client;

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub struct PostHeader {
    id: Uuid,
    posted: DateTime<Utc>,
    title: Box<str>,
}

#[derive(Serialize, Deserialize)]
#[derive(Clone, Debug)]
pub struct Post {
    #[serde(flatten)]
    header: PostHeader,
    content: Box<str>,
}

pub mod server {
    use super::Uuid;

    pub enum Message {
        Connected(Uuid),
        Disconnected(Uuid),
        Broadcast(super::client::Message),
    }
}
