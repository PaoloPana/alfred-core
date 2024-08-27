use zeromq::ZmqError;

#[derive(Debug)]
#[derive(thiserror::Error)]
#[from(std::error::Error)]
pub enum Error {
    #[error("Error on connection")]
    ConnectionError,
    #[error("Error publishing {1} in topic {0}")]
    PublishError(String, String),
    #[error("Error subscribing to topic {0}")]
    SubscribeError(String),
    #[error("Error during get_message")]
    GetMessageError,
    #[error("Error converting message")]
    ConversionError,
    #[error("No response topic found")]
    ReplyError,
    #[error("MessageCompressionError: {0}")]
    MessageCompressionError(String),
    #[error("Missing env property: {0}")]
    MissingEnvPropertyError(String),
    #[error("Missing file property: {0}")]
    MissingFilePropertyError(String),
    #[error("ZmqError: {0}")]
    ZmqError(ZmqError),
}

impl From<MessageCompressionError> for Error {
    fn from(value: MessageCompressionError) -> Self {
        Error::MessageCompressionError(value.to_string())
    }
}

impl From<ZmqError> for Error {
    fn from(value: ZmqError) -> Self {
        Error::ZmqError(value)
    }
}

#[derive(Debug)]
#[derive(thiserror::Error)]
pub enum MessageCompressionError{
    #[error("field {0} not found!")]
    FieldNotFound(String),
    #[error("message type {0} not found!")]
    MessageType(String)
}