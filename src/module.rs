use std::future::Future;
use crate::config::Config;
use crate::connection::{Connection, Publisher, Subscriber};
use crate::error::Error;
use crate::message::Message;

pub struct Module {
    pub module_name: String,
    pub config: Config,
    pub connection: Connection
}

impl Module {
    pub async fn new(module_name: String) -> Result<Module, Error> {
        let config = Config::read()?;
        let connection = Connection::new(&config).await?;
        Ok(Module { module_name, config, connection })
    }
}

impl Subscriber for Module {
    fn subscribe(&mut self, topic: String) -> impl Future<Output=Result<(), Error>> {
        self.connection.subscriber.subscribe(topic)
    }

    fn subscribe_all(&mut self, topics: Vec<String>) -> impl Future<Output=Result<(), Error>> {
        self.connection.subscriber.subscribe_all(topics)
    }

    async fn get_message(&mut self) -> Result<(String, Message), Error> {
        let mut received = false;
        let mut topic= "".to_string();
        let mut message: Message = Message::empty();
        while !received {
            (topic, message) = self.connection.subscriber.get_message().await?;
            received = !self.connection.manage_module_info_request(topic.clone(), self.module_name.clone()).await?;
        }
        Ok((topic, message))
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