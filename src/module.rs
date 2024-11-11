use std::collections::HashMap;
use crate::config::Config;
use crate::error::Error;
use crate::message::{Message, MessageType};
use crate::connection::{Connection, MODULE_INFO_TOPIC_REQUEST, MODULE_INFO_TOPIC_RESPONSE, TOPIC_PREFIX};

pub struct AlfredModule {
    pub module_name: String,
    pub config: Config,
    pub connection: Connection,
    pub capabilities: HashMap<String, String>
}

impl AlfredModule {

    pub async fn new(module_name: &str) -> Result<Self, Error> {
        Self::new_with_details(module_name, None, None).await
    }

    pub async fn new_with_details(module_name: &str, config: Option<Config>, capabilities: Option<HashMap<String, String>>) -> Result<Self, Error> {
        let config = config.unwrap_or_else(|| Config::read(Some(module_name)));
        let capabilities = capabilities.unwrap_or_default();
        let mut connection = Connection::new(&config).await?;
        connection.listen(MODULE_INFO_TOPIC_REQUEST).await?;
        let module_name = module_name.to_string();
        let alfred_module = Self { module_name, config, connection, capabilities };
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

    pub async fn listen(&mut self, topic: &str) -> Result<(), Error> {
        self.capabilities.insert(String::from(TOPIC_PREFIX), String::from(topic));
        self.connection.listen(topic).await
    }

    pub async fn receive(&self) -> Result<(String, Message), Error> {
        let mut received = false;
        let mut topic = String::new();
        let mut message = Message::empty();
        while !received {
            (topic, message) = self.connection.receive().await?;
            received = !self.connection.manage_module_info_request(topic.as_str(), self.module_name.as_str(), &self.capabilities).await?;
        }
        Ok((topic, message))
    }

    pub async fn send(&self, topic: &str, message: &Message) -> Result<(), Error> {
        self.connection.send(topic, message).await
    }

    pub async fn send_event(&mut self, publisher_name: &str, event_name: &str, message: &Message) -> Result<(), Error> {
        self.connection.send_event(publisher_name, event_name, message).await
    }
}