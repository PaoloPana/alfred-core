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

    pub(crate) async fn listen(&mut self, topic: &str) -> Result<(), Error> {
        debug!("Subscribing to topic {topic}");
        self.subscriber.subscribe(topic).await.map_err(|_| Error::SubscribeError(topic.to_string()))
    }

    pub(crate) async fn receive(&mut self) -> Result<(String, Message), Error> {
        let zmq_message = self.subscriber.recv().await.map_err(|_| Error::GetMessageError)?;
        debug!("New message received.");
        let frames = zmq_message.into_vec();
        let topic_frame = frames.first().ok_or(Error::GetMessageError)?;
        let topic_string = String::from_utf8(topic_frame.to_vec()).or(Err(Error::ConversionError))?;

        let message = Message::decode(frames.get(1..).unwrap_or_default())?;
        debug!("{topic_string}: {message:?}");
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

    pub(crate) async fn send(&mut self, topic: &str, message: &Message) -> Result<(), Error> {
        debug!("Publishing message to topic {topic}: {message:?}");
        let [header, payload] = message.encode()?;
        let topic_bytes = Bytes::from(topic.to_string());
        let zmq_message: ZmqMessage = vec![topic_bytes, header, payload].try_into().or(Err(Error::ConversionError))?;
        self.publisher.send(zmq_message).await.map_err(|_| Error::PublishError(topic.to_string()))
    }
}
