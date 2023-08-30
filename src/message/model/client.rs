use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[non_exhaustive]
#[rustfmt::skip]
#[derive(Clone, Debug)]
#[derive(Deserialize, Serialize)]
#[serde(tag = "method", content = "content")]
pub enum MessageBody {
    Query(Box<[u8]>),
    Response {
        target: Uuid,
        body: Box<[u8]>
    },
    Get(Uuid),
    Exists(Uuid),
}

#[rustfmt::skip]
#[derive(Builder, Clone, Debug)]
#[derive(Deserialize, Serialize)]
pub struct Message {
    sender: Uuid,

    #[builder(setter(into), default = "::std::time::SystemTime::now().into()")]
    time: DateTime<Utc>,
    body: MessageBody,
}

impl Message {
    pub fn body_length(&self) -> usize {
        use MessageBody::*;
        match &self.body {
            Query(body) | Response { body, .. } => body.len(),
            Get(uuid) | Exists(uuid) => 16,
        }
    }

    pub fn body(&self) -> &MessageBody {
        &self.body
    }
}
