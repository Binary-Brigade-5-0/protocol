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
    Query(Box<str>),
    Response {
        target: Uuid,
        body: Box<str>
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
        use MessageBody as M;
        if let M::Query(body) | M::Response { body, .. } = &self.body {
            body.len()
        } else {
            16
        }
    }

    pub fn body(&self) -> &MessageBody {
        &self.body
    }
}
