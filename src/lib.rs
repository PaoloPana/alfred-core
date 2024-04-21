use std::fmt::Error;
use std::future::Future;
use std::time::Duration;

use bytes::Bytes;
use zeromq::{Socket, SocketRecv, SocketSend, ZmqError};

use crate::message::{Message, MessageCompressionError};

pub mod message;

// TODO: change them with configuration file
pub const LOCAL_URL: &str = "tcp://127.0.0.1";
pub const PUB_PORT: u32 = 5678;
pub const SUB_PORT: u32 = 1234;

pub trait Subscriber {
    fn subscribe(&mut self, topic: String) -> impl Future<Output = Result<(), ZmqError>>;
    fn get_message(&mut self) -> impl Future<Output = Result<Message, MessageCompressionError>>;
}

pub trait Publisher {
    fn publish_str(&mut self, topic: String, message: String) -> impl Future<Output = Result<(), Error>>;
    fn publish(&mut self, topic: String, message: &Message) -> impl Future<Output = Result<(), Error>>;
}

pub struct Connection {
    subscriber: zeromq::SubSocket,//async_zmq::Subscribe,
    pub publisher: zeromq::PubSocket
}
impl Connection {
    fn get_string_connection(url: String, port: u32) -> String {
        format!("{}:{}", url, port)
    }

    pub async fn new() -> Result<Connection, Error> {
        //let subscriber = async_zmq::subscribe(Self::get_string_connection(LOCAL_URL.to_string(), SUB_PORT).as_str())?.connect()?;
        let mut subscriber = zeromq::SubSocket::new();
        subscriber.connect(Self::get_string_connection(LOCAL_URL.to_string(), SUB_PORT).as_str())
            .await
            .expect("Error on subscriber connection");
        let mut publisher = zeromq::PubSocket::new();
        publisher.connect(Self::get_string_connection(LOCAL_URL.to_string(), PUB_PORT).as_str())
            .await
            .expect("Error on publisher connection");
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(Connection { subscriber, publisher })
    }
}

impl Subscriber for Connection {

    async fn subscribe(&mut self, topic: String) -> Result<(), ZmqError> {
        return self.subscriber.subscribe(topic.as_str()).await;
    }

    async fn get_message(&mut self) -> Result<Message, MessageCompressionError> {
        let zmq_message = self.subscriber.recv().await.expect("Error during get_message");
        let topic_string = String::from_utf8(zmq_message.get(0).unwrap().to_vec()).expect("Error on topic conversion");
        let msg_string = String::from_utf8(zmq_message.get(1).unwrap().to_vec()).expect("Error on message conversion");
        Message::decompress(msg_string.as_str())
    }
}

impl Publisher for Connection {
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
