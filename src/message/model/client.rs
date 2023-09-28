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

    _state: std::marker::PhantomData<T>,
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

            _state: Default::default(),
        }
    }
}

impl<T: BuilderState> MessageBuilder<T> {
    pub fn sender(&mut self, uuid: Uuid) -> &mut Self {
        self.sender = uuid;
        self
    }

    pub fn body(self, body: MessageBody) -> MessageBuilder<Full> {
        let Self { sender, time, .. } = self;

        MessageBuilder {
            sender,
            time,
            body: Full(body),

            _state: Default::default(),
        }
    }

    pub fn time(&mut self, time: SystemTime) -> &mut Self {
        self.time = time;
        self
    }
}

impl MessageBuilder<Full> {
    pub fn build(self) -> Message {
        Message {
            time: self.time.into(),
            sender: self.sender,
            body: self.body.0,
        }
    }
}
