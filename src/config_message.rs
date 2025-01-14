use std::collections::LinkedList;
use serde_derive::Deserialize;
use crate::message::{Message, MessageType};

#[derive(Deserialize, Clone, Debug)]
pub struct ConfigMessage {
    pub text: Option<String>,
    pub response_topics: Option<LinkedList<String>>,
    pub sender: Option<String>,
    pub message_type: Option<MessageType>,
    // TODO: implement pub params: Option<BTreeMap<String, String, RandomState>>,
}

impl ConfigMessage {
    pub fn generate_message(&self, default: &Message) -> Message {
        Message {
            text: self.text.clone().unwrap_or_else(|| default.text.clone()),
            response_topics: self.response_topics.clone().unwrap_or_else(|| default.response_topics.clone()),
            sender: self.sender.clone().unwrap_or_else(|| default.sender.clone()),
            message_type: self.message_type.clone().unwrap_or_else(|| default.message_type.clone()),
            params: default.params.clone(),
        }
    }
}