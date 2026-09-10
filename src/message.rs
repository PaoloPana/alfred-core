use std::{fmt, str::FromStr};
use std::collections::{BTreeMap, LinkedList};
use std::fmt::Debug;
use itertools::Itertools;
use serde_derive::Deserialize;
use crate::error::MessageCompressionError;

const MESSAGE_SEPARATOR : char = 0x0 as char;

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
    pub const fn compress(&self) -> char {
        match self {
            Self::Unknown => 0x00 as char,
            Self::Text => 0x01 as char,
            Self::Audio => 0x2 as char,
            Self::Photo => 0x03 as char,
            Self::StreamText => 0x81 as char,
            Self::StreamAudio => 0x82 as char,
            Self::StreamPhoto => 0x83 as char,
            Self::ModuleInfo => 0xFF as char,
        }
    }

    pub fn decompress(val: char) -> Result<Self, String> {
        let u8_val = val as u8;
        Ok(match u8_val {
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

#[derive(PartialEq, Eq, Default)]
pub struct Message {
    pub message_type: MessageType,
    /// For `Stream*` message types, whether this is the last chunk of the stream.
    /// Ignored for non-stream message types.
    pub is_final: bool,
    /// For `Stream*` message types, an id shared by every chunk of the same stream, letting a
    /// receiver tell chunks of concurrently in-flight streams apart. Ignored for non-stream
    /// message types.
    pub stream_id: String,
    /// For `Stream*` message types, the position of this chunk within its stream (0-based),
    /// letting a receiver detect gaps or reorder chunks that arrive out of order. Ignored for
    /// non-stream message types.
    pub sequence: u32,
    pub params: BTreeMap<String, String>,
    pub response_topics: LinkedList<String>,
    pub sender: String,
    pub text: String,
}

impl Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}][is_final: {}][stream_id: {}][sequence: {}] {}\n - params: {:?}\n - response_topics: {:?}",
               self.message_type,
               self.is_final,
               self.stream_id,
               self.sequence,
               self.text,
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
            text: self.text.clone(),
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
///     text: String::from("text"),
///     response_topics: LinkedList::from([String::from("module.response"), String::from("other.module")]),
///     sender: String::from("0123"),
///     message_type: MessageType::Text,
///     is_final: false,
///     stream_id: String::new(),
///     sequence: 0,
///     params
/// };
/// let compress = message.compress();
/// let result = Message::decompress(compress.as_str()).unwrap();
/// assert_eq!(message, result);
/// ```
impl Message {

    pub fn empty() -> Self {
        Self::default()
    }

    /// compress
    /// # Examples
    /// ```rust
    /// use std::collections::{BTreeMap, LinkedList, VecDeque};
    /// use std::io::Lines;
    /// use alfred_core::message::{Message, MessageType};
    ///
    /// const MESSAGE_SEPARATOR : char = 0x0 as char;
    ///
    /// let message: Message = Message {
    ///     message_type: MessageType::Text,
    ///     is_final: false,
    ///     stream_id: String::new(),
    ///     sequence: 0,
    ///     text: String::from("data"),
    ///     response_topics: LinkedList::from([String::from("module.response"), String::from("other.module")]),
    ///     sender: String::from("0123"),
    ///     params: BTreeMap::from([
    ///         (String::from("par1"), String::from("val1")),
    ///         (String::from("par2"), String::from("val2"))
    ///     ])
    /// };
    /// let compressed = message.compress();
    /// let expected_message_type = 0x1 as char;
    /// let expected_is_final = 0x0 as char;
    /// let expected_num_params = 0x2 as char;
    /// let expected_num_responses = 0x2 as char;
    /// let expected = format!("{expected_message_type}{expected_is_final}{expected_num_params}{expected_num_responses}par1{MESSAGE_SEPARATOR}val1{MESSAGE_SEPARATOR}par2{MESSAGE_SEPARATOR}val2{MESSAGE_SEPARATOR}module.response{MESSAGE_SEPARATOR}other.module{MESSAGE_SEPARATOR}0123{MESSAGE_SEPARATOR}{MESSAGE_SEPARATOR}0{MESSAGE_SEPARATOR}data");
    /// assert_eq!(compressed, expected);
    /// ```
    pub fn compress(&self) -> String {

        #[allow(clippy::cast_possible_truncation)]
        let compress_number = |val: usize| {
            char::from(val as u8).to_string()
        };

        let params = self.params.iter()
            .map(|(k, v)| format!("{k}{MESSAGE_SEPARATOR}{v}{MESSAGE_SEPARATOR}"))
            .join("");
        let response_topics = self.response_topics.iter()
            .map(|val| format!("{val}{MESSAGE_SEPARATOR}"))
            .join("");

        [
            self.message_type.compress().to_string(),
            char::from(u8::from(self.is_final)).to_string(),
            compress_number(self.params.len()),
            compress_number(self.response_topics.len()),
            params,
            response_topics,
            self.sender.clone(),
            MESSAGE_SEPARATOR.to_string(),
            self.stream_id.clone(),
            MESSAGE_SEPARATOR.to_string(),
            self.sequence.to_string(),
            MESSAGE_SEPARATOR.to_string(),
            self.text.clone(),
        ]
            .into_iter()
            .collect::<String>()
    }

    /// decompress
    /// # Examples
    /// ```rust
    /// use std::collections::{BTreeMap, LinkedList, VecDeque};
    /// use std::io::Lines;
    /// use alfred_core::message::{Message, MessageType};
    ///
    /// const MESSAGE_SEPARATOR : char = 0x0 as char;
    ///
    /// let compressed_message_type = 0x01 as char;
    /// let compressed_is_final = 0x00 as char;
    /// let compressed_num_params = 0x02 as char;
    /// let compressed_num_responses = 0x02 as char;
    /// let decompressed = Message::decompress(format!("{compressed_message_type}{compressed_is_final}{compressed_num_params}{compressed_num_responses}par1{MESSAGE_SEPARATOR}val1{MESSAGE_SEPARATOR}par2{MESSAGE_SEPARATOR}val2{MESSAGE_SEPARATOR}module.response{MESSAGE_SEPARATOR}other.module{MESSAGE_SEPARATOR}0123{MESSAGE_SEPARATOR}{MESSAGE_SEPARATOR}0{MESSAGE_SEPARATOR}data").as_str());
    /// assert!(decompressed.is_ok());
    /// let message: Message = Message {
    ///     message_type: MessageType::Text,
    ///     is_final: false,
    ///     stream_id: String::new(),
    ///     sequence: 0,
    ///     text: String::from("data"),
    ///     response_topics: LinkedList::from([String::from("module.response"), String::from("other.module")]),
    ///     sender: String::from("0123"),
    ///     params: BTreeMap::from([
    ///         (String::from("par1"), String::from("val1")),
    ///         (String::from("par2"), String::from("val2"))
    ///     ])
    /// };
    /// assert_eq!(message, decompressed.unwrap());
    /// ```
    /// ```rust
    /// use std::collections::{BTreeMap, LinkedList, VecDeque};
    /// use std::io::Lines;
    /// use alfred_core::message::{Message, MessageType};
    ///
    /// const MESSAGE_SEPARATOR : char = 0x0 as char;
    ///
    /// let compressed_message_type = 0x81 as char; // StreamText
    /// let compressed_is_final = 0x01 as char;
    /// let compressed_num_params = 0x02 as char;
    /// let compressed_num_responses = 0x00 as char;
    /// let decompressed = Message::decompress(format!("{compressed_message_type}{compressed_is_final}{compressed_num_params}{compressed_num_responses}par1{MESSAGE_SEPARATOR}val1{MESSAGE_SEPARATOR}par2{MESSAGE_SEPARATOR}val2{MESSAGE_SEPARATOR}0123{MESSAGE_SEPARATOR}stream-42{MESSAGE_SEPARATOR}7{MESSAGE_SEPARATOR}data").as_str());
    /// assert!(decompressed.is_ok());
    /// let message: Message = Message {
    ///     message_type: MessageType::StreamText,
    ///     is_final: true,
    ///     stream_id: String::from("stream-42"),
    ///     sequence: 7,
    ///     text: String::from("data"),
    ///     response_topics: LinkedList::from([]),
    ///     sender: String::from("0123"),
    ///     params: BTreeMap::from([
    ///         (String::from("par1"), String::from("val1")),
    ///         (String::from("par2"), String::from("val2"))
    ///     ])
    /// };
    /// assert_eq!(message, decompressed.unwrap());
    /// ```
    pub fn decompress(comp_str: &str) -> Result<Self, MessageCompressionError> {
        let mut chars = comp_str.chars();
        let mut get_next_char = || {
            chars.next().ok_or_else(|| MessageCompressionError::DecompressionError(format!("No chars left: {comp_str}")))
        };

        let message_type_str = get_next_char()?;
        let message_type = MessageType::decompress(message_type_str)
            .map_err(|_| MessageCompressionError::DecompressionError(format!("Unknown message_type {message_type_str}: {comp_str}")))?;

        let is_final = (get_next_char()? as u8) != 0;

        let mut offset = 0;
        let params_size = (get_next_char()? as u8) as usize;
        let response_topics_size = (get_next_char()? as u8) as usize;

        // Collect the remaining chars (rather than byte-slicing comp_str) since a message_type
        // or is_final char above 0x7F encodes as more than one byte in UTF-8.
        let rest = chars.collect::<String>();
        let ser_msg = rest.split(MESSAGE_SEPARATOR).collect::<Vec<&str>>();

        let mut params: BTreeMap<String, String> = BTreeMap::new();
        for index in 0..params_size {
            params.insert(ser_msg[2*index].to_string(), ser_msg[2*index+1].to_string());
        }
        offset += 2*params_size;

        let mut response_topics= LinkedList::new();
        for index in 0..response_topics_size {
            response_topics.push_back(ser_msg[offset + index].to_string());
        }
        offset += response_topics_size;

        let sender = ser_msg[offset].to_string();
        let stream_id = ser_msg[offset + 1].to_string();
        let sequence = ser_msg[offset + 2].parse::<u32>()
            .map_err(|_| MessageCompressionError::DecompressionError(format!("Invalid sequence {}: {comp_str}", ser_msg[offset + 2])))?;
        let text = ser_msg[offset + 3..].join(MESSAGE_SEPARATOR.to_string().as_str());

        Ok(Self {
            message_type,
            is_final,
            stream_id,
            sequence,
            params,
            response_topics,
            sender,
            text,
        })
    }

    pub fn reply(&self, text: String, message_type: MessageType) -> Result<(String, Self), crate::error::Error> {
        self.reply_chunk(text, message_type, true, String::new(), 0)
    }

    pub fn reply_chunk(&self, text: String, message_type: MessageType, is_final: bool, stream_id: String, sequence: u32) -> Result<(String, Self), crate::error::Error> {
        let mut response_topics = self.response_topics.clone();
        let topic = response_topics.pop_front().ok_or(crate::error::Error::ReplyError)?;
        let response = Self {
            text,
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

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.compress())
    }
}

#[cfg(test)]
mod tests {
    use super::{Message, MessageType};
    use std::collections::{BTreeMap, LinkedList};
    use std::str::FromStr;

    const ALL_TYPES: [MessageType; 8] = [
        MessageType::Unknown,
        MessageType::Text,
        MessageType::Audio,
        MessageType::Photo,
        MessageType::StreamText,
        MessageType::StreamAudio,
        MessageType::StreamPhoto,
        MessageType::ModuleInfo,
    ];

    #[test]
    fn compress_decompress_round_trip() {
        for message_type in &ALL_TYPES {
            let compressed = message_type.compress();
            let decompressed = MessageType::decompress(compressed).expect("compressed value should decompress");
            assert_eq!(*message_type, decompressed);
        }
    }

    #[test]
    fn display_from_str_round_trip() {
        for message_type in &ALL_TYPES {
            let displayed = message_type.to_string();
            let parsed = MessageType::from_str(&displayed).expect("displayed value should parse");
            assert_eq!(*message_type, parsed);
        }
    }

    #[test]
    fn stream_types_are_distinct_from_base_types() {
        assert_ne!(MessageType::StreamText.compress(), MessageType::Text.compress());
        assert_ne!(MessageType::StreamAudio.compress(), MessageType::Audio.compress());
        assert_ne!(MessageType::StreamPhoto.compress(), MessageType::Photo.compress());
    }

    #[test]
    fn message_round_trip_for_every_type_and_is_final() {
        for message_type in &ALL_TYPES {
            for is_final in [false, true] {
                let message = Message {
                    message_type: message_type.clone(),
                    is_final,
                    stream_id: String::from("stream-abc"),
                    sequence: 42,
                    text: String::from("data"),
                    response_topics: LinkedList::from([String::from("a.b"), String::from("c.d")]),
                    sender: String::from("0123"),
                    params: BTreeMap::from([(String::from("par1"), String::from("val1"))]),
                };
                let compressed = message.compress();
                let decompressed = Message::decompress(&compressed).expect("compressed message should decompress");
                assert_eq!(message, decompressed);
            }
        }
    }

    #[test]
    fn reply_chunk_sets_stream_metadata() {
        let request = Message {
            response_topics: LinkedList::from([String::from("next.topic")]),
            ..Message::default()
        };
        let (topic, response) = request.reply_chunk(
            String::from("chunk"),
            MessageType::StreamText,
            true,
            String::from("stream-1"),
            3,
        ).expect("reply_chunk should succeed when a response topic is present");
        assert_eq!(topic, "next.topic");
        assert_eq!(response.stream_id, "stream-1");
        assert_eq!(response.sequence, 3);
        assert!(response.is_final);
    }

    #[test]
    fn reply_defaults_stream_metadata() {
        let request = Message {
            response_topics: LinkedList::from([String::from("next.topic")]),
            ..Message::default()
        };
        let (_, response) = request.reply(String::from("text"), MessageType::Text)
            .expect("reply should succeed when a response topic is present");
        assert_eq!(response.stream_id, "");
        assert_eq!(response.sequence, 0);
    }
}
