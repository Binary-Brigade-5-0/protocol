#![cfg_attr(debug_assertions, allow(dead_code))]

pub mod drain;
pub mod drained;
pub mod duplex;

#[non_exhaustive]
pub struct Sender;
#[non_exhaustive]
pub struct Receiver;

pub trait MailboxType {}

impl MailboxType for Sender {}
impl MailboxType for Receiver {}
