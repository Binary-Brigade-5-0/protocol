use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Post, PostHeader as Pheader};

pub trait BuilderState {}
pub struct Full(MessageBody);
pub struct Partial;

impl BuilderState for Full {}
impl BuilderState for Partial {}

#[non_exhaustive]
#[derive(Clone, Debug)]
#[derive(Deserialize, Serialize)]
#[serde(tag = "method", content = "body")]
pub enum MessageBody {
    Query(Box<str>),
    Connected(Uuid),

    Error { criminal: Uuid, error: Box<str> },
    Response { target: Uuid, posts: Vec<Pheader> },
    Post { target: Uuid, post: Post },
    Get { target: Uuid, id: Uuid },
}

#[derive(Clone, Debug)]
#[derive(Deserialize, Serialize)]
pub struct Message {
    sender: Uuid,
    time: DateTime<Utc>,

    #[serde(flatten)]
    body: MessageBody,
}

pub struct MessageBuilder<T: BuilderState> {
    sender: Uuid,
    time: SystemTime,
    body: T,
}

impl Message {
    pub fn body(&self) -> &MessageBody {
        &self.body
    }

    pub fn sender(&self) -> &Uuid {
        &self.sender
    }

    pub fn time(&self) -> SystemTime {
        self.time.into()
    }
}

impl Message {
    pub fn builder() -> MessageBuilder<Partial> {
        MessageBuilder {
            sender: Uuid::nil(),
            time: SystemTime::now(),
            body: Partial,
        }
    }
}

impl<T: BuilderState> MessageBuilder<T> {
    pub fn sender(self, sender: Uuid) -> Self {
        Self { sender, ..self }
    }

    pub fn time(self, time: SystemTime) -> Self {
        Self { time, ..self }
    }

    pub fn body(self, body: MessageBody) -> MessageBuilder<Full> {
        let Self { sender, time, .. } = self;

        MessageBuilder {
            sender,
            time,
            body: Full(body),
        }
    }
}

impl MessageBuilder<Full> {
    pub fn build(self) -> Message {
        let Self { sender, time, body } = self;
        let time = time.into();
        let Full(body) = body;

        Message { sender, time, body }
    }
}
