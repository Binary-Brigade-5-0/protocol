use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Kind {
    Created,
    Post,
    Request,
    Response,
    Ok,
    Err,
}

#[derive(Clone, Builder, Serialize, Deserialize, Debug)]
pub struct Message {
    pub kind: Kind,
    pub time: DateTime<Utc>,
    pub message: String,
}
