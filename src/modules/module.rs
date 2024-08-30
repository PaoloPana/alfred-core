use std::future::Future;
use crate::connection::{Receiver, Sender};
use crate::error::Error;
use crate::message::Message;

pub const TOPIC_PREFIX: &str = "event";

pub trait Module: Sender + Receiver {

    fn send_event(&mut self, publisher_name: String, event_name: String, message: &Message) -> impl Future<Output = Result<(), Error>> {
        self.send(
            format!("{TOPIC_PREFIX}.{publisher_name}.{event_name}"),
            message
        )
    }
}