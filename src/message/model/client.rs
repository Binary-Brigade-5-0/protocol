use std::time::SystemTime;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait BuilderState {}
pub struct Full;
pub struct Partial;

impl BuilderState for Full {}
impl BuilderState for Partial {}

#[non_exhaustive]
#[derive(Clone, Debug)]
#[derive(Deserialize, Serialize)]
#[serde(tag = "method", content = "content")]
pub enum MessageBody {
    Query(Box<str>),
    Response { target: Uuid, body: Box<str> },
    Get(Uuid),
    Exists(Uuid),
}

#[derive(Clone, Debug)]
#[derive(Deserialize, Serialize)]
pub struct Message {
    sender: Uuid,

    time: DateTime<Utc>,
    body: MessageBody,
}

pub struct MessageBuilder<T: BuilderState> {
    sender: Uuid,
    time: SystemTime,
    body: Option<MessageBody>,

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
            body: None,

            _state: Default::default(),
        }
    }
}

impl<T: BuilderState> MessageBuilder<T> {
    pub fn sender(&mut self, uuid: Uuid) -> &mut Self {
        self.sender = uuid;
        self
    }

    pub fn body(mut self, body: MessageBody) -> MessageBuilder<Full> {
        self.body = Some(body);

        MessageBuilder {
            sender: self.sender,
            time: self.time,
            body: self.body,

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
            body: self.body.unwrap(),

            sender: self.sender,
        }
    }
}
