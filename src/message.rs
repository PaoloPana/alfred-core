use std::{fmt, str::FromStr, collections::{HashMap, hash_map::RandomState}};
use std::collections::LinkedList;
use std::fmt::Error;
use itertools::Itertools;

const MESSAGE_SEPARATOR : char = 0x0 as char;
const VEC_SEPARATOR : char = 0xFF as char;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum MessageType {
    UNKNOWN,
    TEXT,
    AUDIO,
    PHOTO
}

impl FromStr for MessageType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "UNKNOWN" => MessageType::UNKNOWN,
            "TEXT" => MessageType::TEXT,
            "AUDIO" => MessageType::AUDIO,
            "PHOTO" => MessageType::PHOTO,
            _ => Err(format!("{} is not a valid MessageType.", s))?
        })
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            MessageType::UNKNOWN => "UNKNOWN",
            MessageType::TEXT => "TEXT",
            MessageType::AUDIO => "AUDIO",
            MessageType::PHOTO => "PHOTO",
        })
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Message {
    pub text: String,
    pub starting_module: String,
    pub request_topic: String,
    pub response_topics: LinkedList<String>,
    pub sender: String,
    pub message_type: MessageType,
    pub params: HashMap<String, String, RandomState>,
}

impl Default for Message {
    fn default() -> Self {
        return Self {
            text: String::from(""),
            starting_module: String::from(""),
            request_topic: String::from(""),
            response_topics: LinkedList::new(),
            sender: String::from(""),
            message_type: MessageType::UNKNOWN,
            params: HashMap::new()
        };
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

impl Message {

    pub fn empty() -> Self {
        return Message::default();
    }

    pub fn compress(&self) -> String {
        let params = self.params.iter()
            .map(|(k, v)| format!("{k}{MESSAGE_SEPARATOR}{v}"));

        return
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
                .join(MESSAGE_SEPARATOR.to_string().as_str()).clone();
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
    /// let decompressed = Message::decompress(format!("ciao{MESSAGE_SEPARATOR}telegram{MESSAGE_SEPARATOR}user.request{MESSAGE_SEPARATOR}telegram.response{MESSAGE_SEPARATOR}0123{MESSAGE_SEPARATOR}TEXT{MESSAGE_SEPARATOR}par1{MESSAGE_SEPARATOR}val1{MESSAGE_SEPARATOR}par2{MESSAGE_SEPARATOR}val2{MESSAGE_SEPARATOR}").as_str());
    /// assert!(decompressed.is_ok());
    /// let mut params = HashMap::new();
    /// params.insert(String::from("par1"), String::from("val1"));
    /// params.insert(String::from("par2"), String::from("val2"));
    /// let message: Message = Message {
    ///     text: String::from("ciao"),
    ///     starting_module: String::from("telegram"),
    ///     request_topic: String::from("user.request"),
    ///     response_topics: LinkedList::from(vec!(String::from("telegram.response"))),
    ///     sender: String::from("0123"),
    ///     message_type: MessageType::TEXT,
    ///     params
    /// };
    /// assert_eq!(message, decompressed.unwrap());
    /// ```
    pub fn decompress(comp_str: &str) -> Result<Self, MessageCompressionError> {

        let binding = comp_str.to_string();
        let ser_msg = binding.split(MESSAGE_SEPARATOR).collect::<Vec<&str>>();

        let get_field = |index :usize, field_name: &str| {
            ser_msg
                .get(index)
                .ok_or(MessageCompressionError::FieldNotFound(field_name.to_string()))
        };

        let get_vec = |index :usize, field_name: &str| {
            Ok(ser_msg
                .get(index)
                .ok_or(MessageCompressionError::FieldNotFound(field_name.to_string()))?
                .split(VEC_SEPARATOR)
                .map(|val| val.to_string())
                .collect::<LinkedList<String>>())
        };

        let get_params = |start: usize| {
            let mut param_name = "".to_string();
            let mut params: HashMap<String, String> = HashMap::new();
            for i in start..ser_msg.len() {
                let param = ser_msg.get(i).unwrap().to_string();
                if i % 2 == 0 {
                    param_name = param;
                } else {
                    params.insert(param_name.clone(), param);
                }
            }
            return params;
        };

        return Ok(Message{
            text: get_field(0,"text")?.to_string(),
            starting_module: get_field(1,"starting_module")?.to_string(),
            request_topic: get_field(2,"request_topic")?.to_string(),
            response_topics: get_vec(3,"response_topics")?,
            sender: get_field(4,"sender")?.to_string(),
            message_type: get_field(5,"message_type")?.to_string().parse::<MessageType>().unwrap(),
            params: get_params(6)
        });
    }

    pub fn reply(&mut self, text: String, message_type: MessageType) -> Result<(String, Self), Error> {
        let mut response_topics = self.response_topics.clone();
        let topic = response_topics.pop_front().expect("No response_topic found");
        let response = Message {
            text,
            starting_module: self.starting_module.clone(),
            request_topic: self.request_topic.clone(),
            response_topics,
            sender: self.sender.clone(),
            message_type,
            params: Default::default(),
        };
        Ok((topic, response))
    }

}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.compress())
    }
}
