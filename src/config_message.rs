use std::collections::LinkedList;
use serde_derive::Deserialize;
use crate::message::{Message, MessageType};

#[derive(Deserialize, Clone, Debug)]
pub struct ConfigMessage {
    pub text: Option<String>,
    pub starting_module: Option<String>,
    pub request_topic: Option<String>,
    pub response_topics: Option<LinkedList<String>>,
    pub sender: Option<String>,
    pub message_type: Option<MessageType>,
    // TODO: implement pub params: Option<HashMap<String, String, RandomState>>,
}

impl ConfigMessage {
    pub fn generate_message(&self, default: &Message) -> Message {
        Message {
            text: self.text.clone().unwrap_or_else(|| default.text.clone()),
            starting_module: self.starting_module.clone().unwrap_or_else(|| default.starting_module.clone()),
            request_topic: self.request_topic.clone().unwrap_or_else(|| default.request_topic.clone()),
            response_topics: self.response_topics.clone().unwrap_or_else(|| default.response_topics.clone()),
            sender: self.sender.clone().unwrap_or_else(|| default.sender.clone()),
            message_type: self.message_type.clone().unwrap_or_else(|| default.message_type.clone()),
            params: default.params.clone(),
        }
    }
}