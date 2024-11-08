use std::collections::HashMap;
use std::future::Future;
use crate::config::Config;
use crate::connection::{Receiver, Sender};
use crate::error::Error;
use crate::message::{Message, MessageType};
use crate::MODULE_INFO_TOPIC_RESPONSE;
use crate::pubsub_connection::{PubSubConnection, MODULE_INFO_TOPIC_REQUEST};

const TOPIC_PREFIX: &str = "event";

pub struct AlfredModule {
    pub module_name: String,
    pub config: Config,
    pub connection: PubSubConnection,
    pub capabilities: HashMap<String, String>
}

impl AlfredModule {

    pub async fn new(module_name: &str) -> Result<Self, Error> {
        Self::new_with_details(module_name, None, None).await
    }

    pub async fn new_with_details(module_name: &str, config: Option<Config>, capabilities: Option<HashMap<String, String>>) -> Result<Self, Error> {
        let config = config.unwrap_or_else(|| Config::read(Some(module_name)));
        let capabilities = capabilities.unwrap_or_default();
        let mut connection = PubSubConnection::new(&config).await?;
        connection.listen(MODULE_INFO_TOPIC_REQUEST).await?;
        let module_name = module_name.to_string();
        let mut alfred_module = Self { module_name, config, connection, capabilities };
        alfred_module.send(MODULE_INFO_TOPIC_RESPONSE, &alfred_module.get_info_message()).await?;
        Ok(alfred_module)
    }

    pub fn get_info_message(&self) -> Message {
        Message {
            text: self.module_name.clone(),
            message_type: MessageType::ModuleInfo,
            params: self.capabilities.clone(),
            ..Message::default()
        }
    }
}

impl Receiver for AlfredModule {
    fn listen(&mut self, topic: &str) -> impl Future<Output=Result<(), Error>> {
        self.capabilities.insert(String::from(TOPIC_PREFIX), String::from(topic));
        self.connection.subscriber.listen(topic)
    }

    async fn receive(&mut self) -> Result<(String, Message), Error> {
        let mut received = false;
        let mut topic = String::new();
        let mut message = Message::empty();
        while !received {
            (topic, message) = self.connection.subscriber.receive().await?;
            received = !self.connection.manage_module_info_request(topic.as_str(), self.module_name.as_str(), &self.capabilities).await?;
        }
        Ok((topic, message))
    }
}

impl Sender for AlfredModule {
    fn send(&mut self, topic: &str, message: &Message) -> impl Future<Output=Result<(), Error>> {
        self.connection.publisher.send(topic, message)
    }
}