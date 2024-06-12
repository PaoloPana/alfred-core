use std::future::Future;
use crate::error::Error;
use crate::message::Message;

pub trait Receiver {
    fn receive(&mut self) -> impl Future<Output = Result<(String, Message), Error>>;
}

pub trait Sender {
    fn send(&mut self, address: String, message: &Message) -> impl Future<Output = Result<(), Error>>;
}