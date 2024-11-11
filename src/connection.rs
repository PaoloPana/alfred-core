use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use crate::config::Config;
use crate::message::{Message, MessageType};
use crate::error::Error;
use log::debug;
use tokio::sync::Mutex;
use crate::zmq_connection::{AlfredPublisher, AlfredSubscriber};

pub const MODULE_INFO_TOPIC_REQUEST: &str = "module.info.request";
pub const MODULE_INFO_TOPIC_RESPONSE: &str = "module.info.response";
pub const TOPIC_PREFIX: &str = "event";

#[derive(Clone)]
pub struct Connection {
    subscriber: Arc<Mutex<AlfredSubscriber>>,
    publisher: Arc<Mutex<AlfredPublisher>>
}

impl Connection {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let subscriber = AlfredSubscriber::new(config.get_alfred_sub_url().as_str()).await?;
        debug!("Connected as subscriber");
        tokio::time::sleep(Duration::from_secs(1)).await;
        let publisher = AlfredPublisher::new(config.get_alfred_pub_url().as_str()).await?;
        debug!("Connected as publisher");
        let mut connection = Self {
            subscriber: Arc::new(Mutex::new(subscriber)),
            publisher: Arc::new(Mutex::new(publisher))
        };
        connection.listen(MODULE_INFO_TOPIC_REQUEST).await?;
        Ok(connection)
    }

    pub async fn listen(&mut self, topic: &str) -> Result<(), Error> {
        self.subscriber.lock().await.listen(topic).await
    }

    pub async fn receive_all(&self) -> Result<(String, Message), Error> {
        self.subscriber.lock().await.receive().await
    }

    async fn send_module_info(&self, module_name: &str, capabilities: &HashMap<String, String>) -> Result<(), Error> {
        let info_msg = Message {
            text: module_name.to_string(),
            message_type: MessageType::Text,
            params: capabilities.clone(),
            ..Message::default()
        };
        self.send(MODULE_INFO_TOPIC_RESPONSE, &info_msg).await
    }

    pub async fn manage_module_info_request(&self, topic: &str, module_name: &str, capabilities: &HashMap<String, String>) -> Result<bool, Error> {
        if topic != MODULE_INFO_TOPIC_REQUEST { return Ok(false); }
        debug!("Received info request. Replying...");
        self.send_module_info(module_name, capabilities).await?;
        Ok(true)
    }

    pub async fn receive(&self) -> Result<(String, Message), Error> {
        loop {
            let (topic, message) = self.receive_all().await?;
            if self.manage_module_info_request(topic.as_str(), "", &HashMap::new()).await? {
                continue;
            }
            return Ok((topic, message));
        }
    }

    pub async fn send(&self, topic: &str, message: &Message) -> Result<(), Error> {
        self.publisher.lock().await.send(topic, message).await
    }

    pub async fn send_event(&self, publisher_name: &str, event_name: &str, message: &Message) -> Result<(), Error> {
        let topic = format!("{TOPIC_PREFIX}.{publisher_name}.{event_name}");
        let topic_ref: &'static str = Box::leak(topic.into_boxed_str());
        self.send(topic_ref, message).await
    }
}