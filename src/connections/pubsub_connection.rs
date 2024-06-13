use std::future::Future;
use std::time::Duration;
use bytes::Bytes;
use zeromq::{Socket, SocketRecv, SocketSend};
use crate::config::Config;
use crate::message::Message;
use crate::error::Error;
use log::debug;
use crate::connections::connection::{Receiver, Sender};

pub const MODULE_INFO_TOPIC_REQUEST: &str = "module.info.request";
pub const MODULE_INFO_TOPIC_RESPONSE: &str = "module.info.response";
pub const REQUEST_TOPIC: &str = "request";

pub struct AlfredSubscriber {
    subscriber: zeromq::SubSocket,
}

impl AlfredSubscriber {
    pub async fn manage_module_info_request(&mut self, topic: &str, module_name: String, alfred_publisher: &mut AlfredPublisher) -> Result<bool, Error> {
        if topic != MODULE_INFO_TOPIC_REQUEST { return Ok(false); }
        debug!("Received info request. Replying...");
        alfred_publisher.send_module_info(module_name).await?;
        Ok(true)
    }
}

impl Receiver for AlfredSubscriber {
    async fn listen(&mut self, topic: String) -> Result<(), Error> {
        debug!("Subscribing to topic {topic}");
        return self.subscriber.subscribe(topic.as_str()).await.map_err(|_| Error::SubscribeError(topic.clone()));
    }

    async fn receive(&mut self) -> Result<(String, Message), Error> {
        let zmq_message = self.subscriber.recv().await.map_err(|_| Error::GetMessageError)?;
        debug!("New message received.");
        let topic_string = String::from_utf8(zmq_message.get(0).unwrap().to_vec()).map_err(|_| Error::ConversionError)?;
        let msg_string = String::from_utf8(zmq_message.get(1).unwrap().to_vec()).map_err(|_| Error::ConversionError)?;

        let message = Message::decompress(msg_string.as_str())?;
        debug!("{topic_string}: {message}");
        Ok((topic_string, message))
    }
}

pub struct AlfredPublisher {
    publisher: zeromq::PubSocket
}

impl AlfredPublisher {
    pub async fn send_module_info(&mut self, module_name: String) -> Result<(), Error> {
        self.publish_str(MODULE_INFO_TOPIC_RESPONSE.to_string(), module_name).await
    }

    async fn publish_str(&mut self, topic: String, message: String) -> Result<(), Error> {
        let topic_bytes = Bytes::from(topic.clone());
        let message_bytes = Bytes::from(message.clone());
        let zmq_message = vec![topic_bytes, message_bytes].try_into().or(Err(Error::ConversionError))?;
        debug!(" > {topic}: {message}");
        self.publisher.send(zmq_message).await.map_err(|_| Error::PublishError(topic, message))
    }

}

impl Sender for AlfredPublisher {
    async fn send(&mut self, topic: String, message: &Message) -> Result<(), Error> {
        debug!("Publishing message {message} to topic {topic}...");
        self.publish_str(topic, message.compress()).await
    }
}

pub struct PubSubConnection {
    pub subscriber: AlfredSubscriber,
    pub publisher: AlfredPublisher
}

impl PubSubConnection {
    pub async fn new(config: &Config) -> Result<PubSubConnection, Error> {
        let mut subscriber = zeromq::SubSocket::new();
        subscriber.connect(config.get_alfred_sub_url().as_str()).await?;
        debug!("Connected as subscriber");
        let mut publisher = zeromq::PubSocket::new();
        publisher.connect(config.get_alfred_pub_url().as_str()).await?;
        debug!("Connected as publisher");
        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut connection = PubSubConnection { subscriber: AlfredSubscriber { subscriber }, publisher: AlfredPublisher { publisher } };
        connection.listen(MODULE_INFO_TOPIC_REQUEST.to_string()).await?;
        Ok(connection)
    }

    pub async fn manage_module_info_request(&mut self, topic: &str, module_name: String) -> Result<bool, Error> {
        self.subscriber.manage_module_info_request(topic, module_name, &mut self.publisher).await
    }
}

impl Receiver for PubSubConnection {
    fn listen(&mut self, topic: String) -> impl Future<Output=Result<(), Error>> {
        self.subscriber.listen(topic)
    }

    async fn receive(&mut self) -> Result<(String, Message), Error> {
        loop {
            let (topic, message) = self.subscriber.receive().await?;
            if self.manage_module_info_request(topic.as_str(), "".to_string()).await? {
                continue;
            }
            return Ok((topic, message));
        }
    }
}

impl Sender for PubSubConnection {
    fn send(&mut self, topic: String, message: &Message) -> impl Future<Output = Result<(), Error>> {
        self.publisher.send(topic, message)
    }
}