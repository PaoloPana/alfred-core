use bytes::Bytes;
use log::debug;
use zeromq::{Socket, SocketRecv, SocketSend, ZmqMessage};
use crate::error::Error;
use crate::message::Message;

pub struct AlfredSubscriber {
    subscriber: zeromq::SubSocket,
}

impl AlfredSubscriber {

    pub(crate) async fn new(url: &str) -> Result<Self, Error> {
        let mut subscriber = zeromq::SubSocket::new();
        subscriber.connect(url).await?;
        Ok(Self { subscriber })
    }

    fn get_slice_from_message(zmq_message: &ZmqMessage, index: usize) -> Result<String, Error> {
        String::from_utf8(zmq_message.get(index).ok_or(Error::GetMessageError)?.to_vec()).or(Err(Error::ConversionError))
    }

    pub(crate) async fn listen(&mut self, topic: &str) -> Result<(), Error> {
        debug!("Subscribing to topic {topic}");
        self.subscriber.subscribe(topic).await.map_err(|_| Error::SubscribeError(topic.to_string()))
    }

    pub(crate) async fn receive(&mut self) -> Result<(String, Message), Error> {
        let zmq_message = self.subscriber.recv().await.map_err(|_| Error::GetMessageError)?;
        debug!("New message received.");
        let topic_string = Self::get_slice_from_message(&zmq_message, 0)?;
        let msg_string = Self::get_slice_from_message(&zmq_message, 1)?;

        let message = Message::decompress(msg_string.as_str())?;
        debug!("{topic_string}: {message}");
        Ok((topic_string, message))
    }
}

pub struct AlfredPublisher {
    publisher: zeromq::PubSocket
}

impl AlfredPublisher {

    pub(crate) async fn new(url: &str) -> Result<Self, Error> {
        let mut publisher = zeromq::PubSocket::new();
        publisher.connect(url).await?;
        Ok(Self { publisher })
    }

    async fn publish_str(&mut self, topic: &str, message: &str) -> Result<(), Error> {
        let topic_bytes = Bytes::from(topic.to_string());
        let message_bytes = Bytes::from(message.to_string());
        let zmq_message = vec![topic_bytes, message_bytes].try_into().or(Err(Error::ConversionError))?;
        debug!(" > {topic}: {message}");
        self.publisher.send(zmq_message).await.map_err(|_| Error::PublishError(topic.to_string(), message.to_string()))
    }

    pub(crate) async fn send(&mut self, topic: &str, message: &Message) -> Result<(), Error> {
        debug!("Publishing message {message} to topic {topic}...");
        self.publish_str(topic, message.compress().as_str()).await
    }
}
