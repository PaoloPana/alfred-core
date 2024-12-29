use std::{fmt, str::FromStr, collections::{HashMap, hash_map::RandomState}};
use std::collections::LinkedList;
use itertools::Itertools;
use serde_derive::Deserialize;
use crate::error::MessageCompressionError;

const MESSAGE_SEPARATOR : char = 0x0 as char;
const VEC_SEPARATOR : char = 0xFF as char;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Default)]
pub enum MessageType {
    #[default]
    Unknown,
    Text,
    Audio,
    Photo,
    ModuleInfo
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
    pub text: String,
    pub starting_module: String,
    pub request_topic: String,
    pub response_topics: LinkedList<String>,
    pub sender: String,
    pub message_type: MessageType,
    pub params: HashMap<String, String, RandomState>,
}

impl Clone for Message {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            starting_module: self.starting_module.clone(),
            request_topic: self.request_topic.clone(),
            response_topics: self.response_topics.clone(),
            sender: self.sender.clone(),
            message_type: self.message_type.clone(),
            params: self.params.clone(),
        }
    }
}

impl Message {

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn compress(&self) -> String {
        let params = self.params.iter()
            .map(|(k, v)| format!("{k}{MESSAGE_SEPARATOR}{v}"));

        [
            self.text.clone(),
            self.starting_module.clone(),
            self.request_topic.clone(),
            Itertools::intersperse(
                self.response_topics.iter()
                    .cloned(), VEC_SEPARATOR.to_string()
            ).collect(),
            self.sender.clone(),
            self.message_type.to_string()
        ]
            .into_iter()
            .chain(params)
            .collect::<Vec<String>>()
            .join(MESSAGE_SEPARATOR.to_string().as_str())
    }

    /// decompress
    /// # Examples
    /// ```rust
    /// use std::collections::{HashMap, LinkedList, VecDeque};
    /// use std::io::Lines;
    /// use alfred_rs::message::{Message, MessageType};
    ///
    /// const MESSAGE_SEPARATOR : char = 0x0 as char;
    ///
    /// let decompressed = Message::decompress(format!("text{MESSAGE_SEPARATOR}module{MESSAGE_SEPARATOR}user.request{MESSAGE_SEPARATOR}module.response{MESSAGE_SEPARATOR}0123{MESSAGE_SEPARATOR}TEXT{MESSAGE_SEPARATOR}par1{MESSAGE_SEPARATOR}val1{MESSAGE_SEPARATOR}par2{MESSAGE_SEPARATOR}val2{MESSAGE_SEPARATOR}").as_str());
    /// assert!(decompressed.is_ok());
    /// let mut params = HashMap::new();
    /// params.insert(String::from("par1"), String::from("val1"));
    /// params.insert(String::from("par2"), String::from("val2"));
    /// let message: Message = Message {
    ///     text: String::from("text"),
    ///     starting_module: String::from("module"),
    ///     request_topic: String::from("user.request"),
    ///     response_topics: LinkedList::from([String::from("module.response")]),
    ///     sender: String::from("0123"),
    ///     message_type: MessageType::Text,
    ///     params
    /// };
    /// assert_eq!(message, decompressed.unwrap());
    /// ```
    pub fn decompress(comp_str: &str) -> Result<Self, MessageCompressionError> {
        let binding = comp_str.to_string();
        let ser_msg = binding.split(MESSAGE_SEPARATOR).collect::<Vec<&str>>();

        let get_field = |index :usize, default: String| {
            ser_msg
                .get(index).copied()
                .map_or(default, ToString::to_string)
        };

        let get_vec = |index :usize, field_name: &str| {
            Ok(ser_msg
                .get(index)
                .ok_or_else(|| MessageCompressionError::FieldNotFound(field_name.to_string()))?
                .split(VEC_SEPARATOR)
                .map(ToString::to_string)
                .collect::<LinkedList<String>>())
        };

        let get_params = |start: usize| {
            let mut param_name = String::new();
            let mut params: HashMap<String, String> = HashMap::new();
            for (i, param) in ser_msg.iter().enumerate().skip(start) {
                if i % 2 == 0 {
                    param_name = (*param).to_string();
                } else {
                    params.insert(param_name.clone(), (*param).to_string());
                }
            }
            params
        };

        Ok(Self{
            text: get_field(0, String::new()),
            starting_module: get_field(1, String::new()),
            request_topic: get_field(2, String::new()),
            response_topics: get_vec(3,"response_topics")?,
            sender: get_field(4, String::new()),
            message_type: get_field(5, MessageType::default().to_string()).parse::<MessageType>()
                .or(Err(MessageCompressionError::DecompressionError()))?,
            params: get_params(6)
        })
    }

    pub fn reply(&self, text: String, message_type: MessageType) -> Result<(String, Self), crate::error::Error> {
        let mut response_topics = self.response_topics.clone();
        let topic = response_topics.pop_front().ok_or(crate::error::Error::ReplyError)?;
        let response = Self {
            text,
            starting_module: self.starting_module.clone(),
            request_topic: self.request_topic.clone(),
            response_topics,
            sender: self.sender.clone(),
            message_type,
            params: HashMap::default(),
        };
        Ok((topic, response))
    }

}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.compress())
    }
}
