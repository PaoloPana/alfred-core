use std::{fmt, str::FromStr};
use std::collections::{BTreeMap, LinkedList};
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
    ModuleInfo
}

impl MessageType {
    pub const fn compress(&self) -> char {
        match self {
            Self::Unknown => 0x00 as char,
            Self::Text => 0x01 as char,
            Self::Audio => 0x2 as char,
            Self::Photo => 0x03 as char,
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
            Self::ModuleInfo => "ModuleInfo"
        })
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Message {
    pub message_type: MessageType,
    pub params: BTreeMap<String, String>,
    pub response_topics: LinkedList<String>,
    pub sender: String,
    pub text: String,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            message_type: self.message_type.clone(),
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
    /// let expected_num_params = 0x2 as char;
    /// let expected_num_responses = 0x2 as char;
    /// let expected = format!("{expected_message_type}{expected_num_params}{expected_num_responses}par1{MESSAGE_SEPARATOR}val1{MESSAGE_SEPARATOR}par2{MESSAGE_SEPARATOR}val2{MESSAGE_SEPARATOR}module.response{MESSAGE_SEPARATOR}other.module{MESSAGE_SEPARATOR}0123{MESSAGE_SEPARATOR}data");
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

        [
            self.message_type.compress().to_string(),
            compress_number(self.params.len()),
            compress_number(self.response_topics.len()),
            params,
            Itertools::intersperse(
                self.response_topics.iter()
                    .cloned(), MESSAGE_SEPARATOR.to_string()
            ).collect(),
            MESSAGE_SEPARATOR.to_string(),
            self.sender.clone(),
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
    /// let compressed_num_params = 0x02 as char;
    /// let compressed_num_responses = 0x02 as char;
    /// let decompressed = Message::decompress(format!("{compressed_message_type}{compressed_num_params}{compressed_num_responses}par1{MESSAGE_SEPARATOR}val1{MESSAGE_SEPARATOR}par2{MESSAGE_SEPARATOR}val2{MESSAGE_SEPARATOR}module.response{MESSAGE_SEPARATOR}other.module{MESSAGE_SEPARATOR}0123{MESSAGE_SEPARATOR}data").as_str());
    /// assert!(decompressed.is_ok());
    /// let message: Message = Message {
    ///     message_type: MessageType::Text,
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
    pub fn decompress(comp_str: &str) -> Result<Self, MessageCompressionError> {
        let mut binding = comp_str.chars();
        let mut get_next_char = || {
            binding.nth(0).ok_or(MessageCompressionError::DecompressionError())
        };

        let message_type_str = get_next_char()?;
        let message_type = MessageType::decompress(message_type_str)
            .map_err(|_| MessageCompressionError::DecompressionError())?;

        let mut offset = 0;
        let params_size = (get_next_char()? as u8) as usize;
        let response_topics_size = (get_next_char()? as u8) as usize;

        let ser_msg = comp_str[3..].split(MESSAGE_SEPARATOR).collect::<Vec<&str>>();

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
        let text = ser_msg[offset + 1..].join(MESSAGE_SEPARATOR.to_string().as_str());

        Ok(Self {
            message_type,
            params,
            response_topics,
            sender,
            text,
        })
    }

    pub fn reply(&self, text: String, message_type: MessageType) -> Result<(String, Self), crate::error::Error> {
        let mut response_topics = self.response_topics.clone();
        let topic = response_topics.pop_front().ok_or(crate::error::Error::ReplyError)?;
        let response = Self {
            text,
            response_topics,
            sender: self.sender.clone(),
            message_type,
            params: BTreeMap::default(),
        };
        Ok((topic, response))
    }

}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.compress())
    }
}
