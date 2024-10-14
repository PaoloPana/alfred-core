use std::future::Future;
use crate::error::Error;
use crate::message::Message;
pub const TOPIC_PREFIX: &str = "event";

pub trait Receiver {
    fn listen(&mut self, address: &str) -> impl Future<Output = Result<(), Error>>;
    fn receive(&mut self) -> impl Future<Output = Result<(String, Message), Error>>;
}

pub trait Sender {
    fn send(&mut self, address: &str, message: &Message) -> impl Future<Output = Result<(), Error>>;
    fn send_event(&mut self, publisher_name: &str, event_name: &str, message: &Message) -> impl Future<Output = Result<(), Error>> {
        let topic = format!("{TOPIC_PREFIX}.{publisher_name}.{event_name}");
        let topic_ref: &'static str = Box::leak(topic.into_boxed_str());
        self.send(
            topic_ref,
            message
        )
    }

}