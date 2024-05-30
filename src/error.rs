#[derive(Debug)]
#[derive(thiserror::Error)]
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
    #[error("MessageCompressionError: {0}")]
    MessageCompressionError(String)
}

impl From<MessageCompressionError> for Error {
    fn from(value: MessageCompressionError) -> Self {
        Error::MessageCompressionError(value.to_string())
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