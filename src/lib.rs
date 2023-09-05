#![cfg_attr(debug_assertions, allow(unused_must_use, unused_mut, unused_variables))]
// TODO: code requires further documentation
// TODO: messaging implementation must be further refined
// TODO: implement proper filtering mechanism
//

// The protocol relies on the fact that atleast some clients (excluding the initial posters) are
// ready to lend their resources for easier distribution of content across the network.
//
// Ensuring this is pretty difficult and is purely a happy-go-lucky situation, since not all
// clients would want to do the forementioned, due to certain clients' resource limitations.

pub mod broker;
pub mod client;
pub mod message;
pub mod server;
