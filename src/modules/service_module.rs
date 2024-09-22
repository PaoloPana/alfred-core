use std::future::Future;
use crate::config::Config;
use crate::pubsub_connection::{PubSubConnection, MODULE_INFO_TOPIC_REQUEST};
use crate::connections::connection::{Receiver, Sender};
use crate::error::Error;
use crate::message::Message;
use crate::modules::module::Module;

pub struct ServiceModule {
    pub module_name: String,
    pub config: Config,
    pub connection: PubSubConnection
}

impl ServiceModule {
    pub async fn new(module_name: String) -> Result<Self, Error> {
        ServiceModule::new_with_custom_config(
            module_name.clone(),
            Config::read(Some(module_name.clone()))?
        ).await
    }

    pub async fn new_with_custom_config(module_name: String, config: Config) -> Result<Self, Error> {
        let mut connection = PubSubConnection::new(&config).await?;
        connection.listen(MODULE_INFO_TOPIC_REQUEST.to_string()).await?;
        Ok(Self { module_name, config, connection })
    }
}

impl Module for ServiceModule {}

impl Receiver for ServiceModule {
    fn listen(&mut self, topic: String) -> impl Future<Output=Result<(), Error>> {
        self.connection.subscriber.listen(topic)
    }

    async fn receive(&mut self) -> Result<(String, Message), Error> {
        let mut received = false;
        let mut topic= "".to_string();
        let mut message = Message::empty();
        while !received {
            (topic, message) = self.connection.subscriber.receive().await?;
            received = !self.connection.manage_module_info_request(topic.as_str(), self.module_name.clone()).await?;
        }
        Ok((topic, message))
    }
}

impl Sender for ServiceModule {
    fn send(&mut self, topic: String, message: &Message) -> impl Future<Output=Result<(), Error>> {
        self.connection.publisher.send(topic, message)
    }
}