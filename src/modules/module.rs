use crate::connection::{Receiver, Sender};

pub const TOPIC_PREFIX: &str = "event";

pub trait Module: Sender + Receiver {
}