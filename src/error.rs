use zeromq::ZmqError;

#[derive(Debug)]
#[derive(thiserror::Error)]
#[from(std::error::Error)]
pub enum Error {
    #[error("Error on connection")]
    ConnectionError,
    #[error("Error publishing in topic {0}")]
    PublishError(String),
    #[error("Error subscribing to topic {0}")]
    SubscribeError(String),
    #[error("Error during get_message")]
    GetMessageError,
    #[error("Error converting message")]
    ConversionError,
    #[error("No response topic found")]
    ReplyError,
    #[error("MessageEncodingError: {0}")]
    MessageEncodingError(String),
    #[error("Payload is not valid UTF-8 text")]
    PayloadNotText,
    #[error("Missing env property: {0}")]
    MissingEnvPropertyError(String),
    #[error("Missing file property: {0}")]
    MissingFilePropertyError(String),
    #[error("ZmqError: {0}")]
    ZmqError(ZmqError),
}

impl From<MessageEncodingError> for Error {
    fn from(value: MessageEncodingError) -> Self {
        Self::MessageEncodingError(value.to_string())
    }
}

impl From<ZmqError> for Error {
    fn from(value: ZmqError) -> Self {
        Self::ZmqError(value)
    }
}

#[derive(Debug)]
#[derive(thiserror::Error)]
pub enum MessageEncodingError{
    #[error("field {0} not found!")]
    FieldNotFound(String),
    #[error("message type {0} not found!")]
    MessageType(String),
    #[error("invalid header: {0}")]
    Header(String),
    #[error("unsupported protocol version {0} (expected {expected})", expected = crate::message::PROTOCOL_VERSION)]
    ProtocolVersion(u8)
}