use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};
use crate::config::Config;
use crate::dedup::DedupCache;
use crate::message::{Message, MessageType};
use crate::error::Error;
use log::{debug, warn};
use crate::mqtt_connection::MqttTransport;

pub const MODULE_INFO_TOPIC_REQUEST: &str = "module.info.request";
pub const MODULE_INFO_TOPIC_RESPONSE: &str = "module.info.response";
pub const TOPIC_PREFIX: &str = "event";
/// Number of recent message ids remembered to drop duplicate deliveries
pub const DEDUP_CACHE_SIZE: usize = 1024;

#[derive(Clone)]
pub struct Connection {
    transport: Arc<MqttTransport>,
    dedup: Arc<Mutex<DedupCache>>,
}

impl Connection {
    pub async fn new(config: &Config) -> Result<Self, Error> {
        let transport = MqttTransport::new(config).await?;
        debug!("Connected to the broker");
        let mut connection = Self {
            transport: Arc::new(transport),
            dedup: Arc::new(Mutex::new(DedupCache::new(DEDUP_CACHE_SIZE))),
        };
        connection.listen(MODULE_INFO_TOPIC_REQUEST).await?;
        Ok(connection)
    }

    /// Listens to `topic` and to its sub-topics: `a.b` receives `a.b` and `a.b.c`, but not `a.bc`.
    /// An empty topic receives everything.
    pub async fn listen(&mut self, topic: &str) -> Result<(), Error> {
        self.transport.listen(topic).await
    }

    /// Receives the next message of any listened topic, dropping the duplicate deliveries of
    /// a message already received.
    pub async fn receive_all(&self) -> Result<(String, Message), Error> {
        loop {
            let (topic, message) = self.transport.receive().await?;
            let duplicate = self.dedup.lock().unwrap_or_else(PoisonError::into_inner).is_duplicate(&message.id);
            if duplicate {
                debug!("Dropping duplicate message {} on {topic}", message.id);
                continue;
            }
            return Ok((topic, message));
        }
    }

    async fn send_module_info(&self, module_name: &str, capabilities: &BTreeMap<String, String>) -> Result<(), Error> {
        let info_msg = Message {
            payload: module_name.to_string().into(),
            message_type: MessageType::Text,
            params: capabilities.clone(),
            ..Message::default()
        };
        self.send(MODULE_INFO_TOPIC_RESPONSE, &info_msg).await
    }

    pub async fn manage_module_info_request(&self, topic: &str, module_name: &str, capabilities: &BTreeMap<String, String>) -> Result<bool, Error> {
        if topic != MODULE_INFO_TOPIC_REQUEST { return Ok(false); }
        debug!("Received info request. Replying...");
        self.send_module_info(module_name, capabilities).await?;
        Ok(true)
    }

    pub async fn receive(&self, module_name: &str, capabilities: &BTreeMap<String, String>) -> Result<(String, Message), Error> {
        loop {
            let (topic, message) = match self.receive_all().await {
                Err(Error::MessageEncodingError(err)) => {
                    warn!("Discarding message that cannot be decoded: {err}");
                    continue;
                },
                received => received?
            };
            if self.manage_module_info_request(topic.as_str(), module_name, capabilities).await? {
                continue;
            }
            return Ok((topic, message));
        }
    }

    /// Publishes `message` on `topic`. A new unique `id` is generated for every call.
    pub async fn send(&self, topic: &str, message: &Message) -> Result<(), Error> {
        self.transport.send(topic, message).await
    }

    pub async fn send_event(&self, publisher_name: &str, event_name: &str, message: &Message) -> Result<(), Error> {
        let topic = format!("{TOPIC_PREFIX}.{publisher_name}.{event_name}");
        self.send(topic.as_str(), message).await
    }

    /// Publishes the offline status and closes the connection cleanly. Dropping the connection
    /// instead closes the socket abruptly: the broker publishes the last will (offline status)
    /// and keeps the session, so the queued messages are delivered at the next start.
    pub async fn disconnect(&self) -> Result<(), Error> {
        self.transport.disconnect().await
    }
}
