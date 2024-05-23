use std::fmt::Error;
use std::future::Future;
use zeromq::ZmqError;
use crate::config::Config;
use crate::connection::{Connection, Publisher, Subscriber};
use crate::message::{Message, MessageCompressionError};

pub struct Module {
    pub module_name: String,
    pub config: Config,
    pub connection: Connection
}

impl Module {
    pub async fn new(module_name: String) -> Module {
        let config = Config::read();
        let connection = Connection::new(&config).await.expect("Error on connection");
        Module {
            module_name,
            config,
            connection,
        }
    }
}

impl Subscriber for Module {
    fn subscribe(&mut self, topic: String) -> impl Future<Output=Result<(), ZmqError>> {
        self.connection.subscriber.subscribe(topic)
    }

    fn subscribe_all(&mut self, topics: Vec<String>) -> impl Future<Output=Result<(), ZmqError>> {
        self.connection.subscriber.subscribe_all(topics)
    }

    fn get_message(&mut self) -> impl Future<Output=Result<(String, Message), MessageCompressionError>> {
        self.connection.subscriber.get_message()
    }
}

impl Publisher for Module {
    fn publish_str(&mut self, topic: String, message: String) -> impl Future<Output=Result<(), Error>> {
        self.connection.publisher.publish_str(topic, message)
    }

    fn publish(&mut self, topic: String, message: &Message) -> impl Future<Output=Result<(), Error>> {
        self.connection.publisher.publish(topic, message)
    }
}