use std::{fmt, str::FromStr};
use std::borrow::Cow;
use std::collections::{BTreeMap, LinkedList};
use std::fmt::Debug;
use bytes::Bytes;
use serde_derive::{Deserialize, Serialize};
use crate::error::{Error, MessageEncodingError};

pub const PROTOCOL_VERSION : u8 = 0x03;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Default)]
pub enum MessageType {
    #[default]
    Unknown,
    Text,
    Audio,
    Photo,
    StreamText,
    StreamAudio,
    StreamPhoto,
    ModuleInfo
}

impl MessageType {
    pub const fn encode(&self) -> u8 {
        match self {
            Self::Unknown => 0x00,
            Self::Text => 0x01,
            Self::Audio => 0x02,
            Self::Photo => 0x03,
            Self::StreamText => 0x81,
            Self::StreamAudio => 0x82,
            Self::StreamPhoto => 0x83,
            Self::ModuleInfo => 0xFF,
        }
    }

    pub fn decode(val: u8) -> Result<Self, String> {
        Ok(match val {
            0x00 => Self::Unknown,
            0x01 => Self::Text,
            0x02 => Self::Audio,
            0x03 => Self::Photo,
            0x81 => Self::StreamText,
            0x82 => Self::StreamAudio,
            0x83 => Self::StreamPhoto,
            0xFF => Self::ModuleInfo,
            _ => Err(format!("{val} is not a valid MessageType."))?
        })
    }
}

impl FromStr for MessageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Unknown" => Self::Unknown,
            "Text" => Self::Text,
            "Audio" => Self::Audio,
            "Photo" => Self::Photo,
            "StreamText" => Self::StreamText,
            "StreamAudio" => Self::StreamAudio,
            "StreamPhoto" => Self::StreamPhoto,
            "ModuleInfo" => Self::ModuleInfo,
            _ => Err(format!("{s} is not a valid MessageType."))?
        })
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Unknown => "Unknown",
            Self::Text => "Text",
            Self::Audio => "Audio",
            Self::Photo => "Photo",
            Self::StreamText => "StreamText",
            Self::StreamAudio => "StreamAudio",
            Self::StreamPhoto => "StreamPhoto",
            Self::ModuleInfo => "ModuleInfo"
        })
    }
}

#[derive(Serialize)]
struct EncodedHeader<'a> {
    message_type: u8,
    is_final: bool,
    stream_id: &'a str,
    sequence: u32,
    params: &'a BTreeMap<String, String>,
    response_topics: Vec<&'a str>,
    sender: &'a str,
}

#[derive(Deserialize)]
struct DecodedHeader {
    message_type: u8,
    is_final: bool,
    stream_id: String,
    sequence: u32,
    params: BTreeMap<String, String>,
    response_topics: Vec<String>,
    sender: String,
}

#[derive(PartialEq, Eq, Default)]
pub struct Message {
    pub message_type: MessageType,
    pub is_final: bool,
    pub stream_id: String,
    pub sequence: u32,
    pub params: BTreeMap<String, String>,
    pub response_topics: LinkedList<String>,
    pub sender: String,
    pub payload: Bytes,
}

impl Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}][is_final: {}][stream_id: {}][sequence: {}] {}\n - params: {:?}\n - response_topics: {:?}",
               self.message_type,
               self.is_final,
               self.stream_id,
               self.sequence,
               self.payload_description(),
               self.params,
               self.response_topics
        )
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            message_type: self.message_type.clone(),
            is_final: self.is_final,
            stream_id: self.stream_id.clone(),
            sequence: self.sequence,
            payload: self.payload.clone(),
            response_topics: self.response_topics.clone(),
            sender: self.sender.clone(),
            params: self.params.clone(),
        }
    }
}

/// Message implementation
/// # Examples
/// ```rust
/// use std::collections::{BTreeMap, LinkedList};
/// use alfred_core::message::{Message, MessageType};
///
/// let params = BTreeMap::from([
///     (String::from("par1"), String::from("val1")),
///     (String::from("par2"), String::from("val2"))
/// ]);
/// let message = Message {
///     payload: "text".into(),
///     response_topics: LinkedList::from([String::from("module.response"), String::from("other.module")]),
///     sender: String::from("0123"),
///     message_type: MessageType::Text,
///     is_final: false,
///     stream_id: String::new(),
///     sequence: 0,
///     params
/// };
/// let frames = message.encode().unwrap();
/// let result = Message::decode(&frames).unwrap();
/// assert_eq!(message, result);
/// assert_eq!(result.text().unwrap(), "text");
/// ```
impl Message {

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn text(&self) -> Result<&str, Error> {
        std::str::from_utf8(&self.payload).map_err(|_| Error::PayloadNotText)
    }

    pub fn payload_description(&self) -> Cow<'_, str> {
        std::str::from_utf8(&self.payload)
            .map_or_else(|_| Cow::Owned(format!("<{} bytes>", self.payload.len())), Cow::Borrowed)
    }

    pub fn encode(&self) -> Result<[Bytes; 2], MessageEncodingError> {
        let header = EncodedHeader {
            message_type: self.message_type.encode(),
            is_final: self.is_final,
            stream_id: &self.stream_id,
            sequence: self.sequence,
            params: &self.params,
            response_topics: self.response_topics.iter().map(String::as_str).collect(),
            sender: &self.sender,
        };
        let mut header_frame = vec![PROTOCOL_VERSION];
        rmp_serde::encode::write_named(&mut header_frame, &header)
            .map_err(|err| MessageEncodingError::Header(err.to_string()))?;
        Ok([Bytes::from(header_frame), self.payload.clone()])
    }

    pub fn decode(frames: &[Bytes]) -> Result<Self, MessageEncodingError> {
        let header_frame = frames.first()
            .ok_or_else(|| MessageEncodingError::FieldNotFound(String::from("header")))?;
        let (version, header) = header_frame.split_first()
            .ok_or_else(|| MessageEncodingError::FieldNotFound(String::from("protocol version")))?;
        if *version != PROTOCOL_VERSION {
            Err(MessageEncodingError::ProtocolVersion(*version))?;
        }
        let payload = frames.get(1)
            .cloned()
            .ok_or_else(|| MessageEncodingError::FieldNotFound(String::from("payload")))?;
        let header: DecodedHeader = rmp_serde::from_slice(header)
            .map_err(|err| MessageEncodingError::Header(err.to_string()))?;
        let message_type = MessageType::decode(header.message_type)
            .map_err(MessageEncodingError::MessageType)?;

        Ok(Self {
            message_type,
            is_final: header.is_final,
            stream_id: header.stream_id,
            sequence: header.sequence,
            params: header.params,
            response_topics: header.response_topics.into_iter().collect(),
            sender: header.sender,
            payload,
        })
    }

    pub fn reply(&self, payload: impl Into<Bytes>, message_type: MessageType) -> Result<(String, Self), Error> {
        self.reply_chunk(payload, message_type, true, String::new(), 0)
    }

    pub fn reply_chunk(&self, payload: impl Into<Bytes>, message_type: MessageType, is_final: bool, stream_id: String, sequence: u32) -> Result<(String, Self), Error> {
        let mut response_topics = self.response_topics.clone();
        let topic = response_topics.pop_front().ok_or(Error::ReplyError)?;
        let response = Self {
            payload: payload.into(),
            response_topics,
            sender: self.sender.clone(),
            message_type,
            is_final,
            stream_id,
            sequence,
            params: self.params.clone(),
        };
        Ok((topic, response))
    }

}
