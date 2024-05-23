use std::fmt::Error;
use std::future::Future;
use std::time::Duration;
use bytes::Bytes;
use zeromq::{Socket, SocketRecv, SocketSend, ZmqError};
use crate::config::Config;
use crate::message::{Message, MessageCompressionError};

pub const MODULE_INFO_TOPIC_REQUEST: &str = "module.info.request";
pub const MODULE_INFO_TOPIC_RESPONSE: &str = "module.info.response";

pub trait Subscriber {
    fn subscribe(&mut self, topic: String) -> impl Future<Output = Result<(), ZmqError>>;
    fn subscribe_all(&mut self, topics: Vec<String>) -> impl Future<Output = Result<(), ZmqError>>;
    fn get_message(&mut self) -> impl Future<Output = Result<(String, Message), MessageCompressionError>>;
}

pub trait Publisher {
    fn publish_str(&mut self, topic: String, message: String) -> impl Future<Output = Result<(), Error>>;
    fn publish(&mut self, topic: String, message: &Message) -> impl Future<Output = Result<(), Error>>;
}

pub struct AlfredSubscriber {
    subscriber: zeromq::SubSocket,
}
pub struct AlfredPublisher {
    publisher: zeromq::PubSocket
}
impl AlfredSubscriber {
    pub async fn manage_module_info_request(&mut self, topic: String, module_name: String, alfred_publisher: &mut AlfredPublisher) -> bool {
        if topic != MODULE_INFO_TOPIC_REQUEST { return false }
        alfred_publisher.publish_str(MODULE_INFO_TOPIC_RESPONSE.to_string(), module_name).await
            .expect(format!("Error during replying on {MODULE_INFO_TOPIC_RESPONSE}").as_str());
        return true
    }
}
impl Subscriber for AlfredSubscriber {

    async fn subscribe(&mut self, topic: String) -> Result<(), ZmqError> {
        return self.subscriber.subscribe(topic.as_str()).await;
    }

    async fn subscribe_all(&mut self, topics: Vec<String>) -> Result<(), ZmqError> {
        for topic in topics {
            self.subscribe(topic).await?;
        }
        Ok(())
    }

    async fn get_message(&mut self) -> Result<(String, Message), MessageCompressionError> {
        let zmq_message = self.subscriber.recv().await.expect("Error during get_message");
        let topic_string = String::from_utf8(zmq_message.get(0).unwrap().to_vec()).expect("Error on topic conversion");
        let msg_string = String::from_utf8(zmq_message.get(1).unwrap().to_vec()).expect("Error on message conversion");

        Ok((topic_string, Message::decompress(msg_string.as_str())?))
    }
}

impl Publisher for AlfredPublisher {
    async fn publish_str(&mut self, topic: String, message: String) -> Result<(), Error> {
        let topic_bytes = Bytes::from(topic);
        let message_bytes = Bytes::from(message);
        let zmq_message = vec![topic_bytes, message_bytes].try_into().expect("Error on conversion");
        Ok(self.publisher.send(zmq_message).await.expect("Error on publish"))
    }

    async fn publish(&mut self, topic: String, message: &Message) -> Result<(), Error> {
        self.publish_str(topic, message.compress()).await
    }
}
pub struct Connection {
    pub subscriber: AlfredSubscriber,
    pub publisher: AlfredPublisher
}
impl Connection {
    pub async fn new(config: &Config) -> Result<Connection, Error> {
        let mut subscriber = zeromq::SubSocket::new();
        subscriber.connect(config.get_alfred_sub_url().as_str()).await
            .expect("Error on subscriber connection");
        let mut publisher = zeromq::PubSocket::new();
        publisher.connect(config.get_alfred_pub_url().as_str()).await
            .expect("Error on publisher connection");
        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut connection = Connection { subscriber: AlfredSubscriber { subscriber }, publisher: AlfredPublisher { publisher } };
        connection.subscribe(MODULE_INFO_TOPIC_REQUEST.to_string())
            .await
            .expect(format!("Error during subscription on {MODULE_INFO_TOPIC_REQUEST}.").as_str());
        Ok(connection)
    }

    pub async fn manage_module_info_request(&mut self, topic: String, module_name: String) -> bool {
        self.subscriber.manage_module_info_request(topic, module_name, &mut self.publisher).await
    }
}
impl Subscriber for Connection {
    fn subscribe(&mut self, topic: String) -> impl Future<Output = Result<(), ZmqError>> {
        self.subscriber.subscribe(topic)
    }

    fn subscribe_all(&mut self, topics: Vec<String>) -> impl Future<Output=Result<(), ZmqError>> {
        self.subscriber.subscribe_all(topics)
    }

    async fn get_message(&mut self) -> Result<(String, Message), MessageCompressionError> {
        loop {
            let (topic, message) = self.subscriber.get_message().await?;
            if self.manage_module_info_request(topic.clone(), "".to_string()).await {
                continue;
            }
            return Ok((topic, message));
        }

    }
}
impl Publisher for Connection {
    fn publish_str(&mut self, topic: String, message: String) -> impl Future<Output = Result<(), Error>> {
        self.publisher.publish_str(topic, message)
    }

    fn publish(&mut self, topic: String, message: &Message) -> impl Future<Output = Result<(), Error>> {
        self.publisher.publish(topic, message)
    }
}