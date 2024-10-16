use std::collections::LinkedList;
use std::error::Error;
use std::fs;
use serde_derive::Deserialize;
use crate::message::{Message, MessageType};

const ROUTING_FILENAME: &str = "routing.toml";

#[derive(Deserialize)]
#[derive(Clone)]
#[derive(Debug)]
pub struct RoutingMessage {
    pub text: Option<String>,
    pub starting_module: Option<String>,
    pub request_topic: Option<String>,
    pub response_topics: Option<LinkedList<String>>,
    pub sender: Option<String>,
    pub message_type: Option<MessageType>,
    // TODO: implement pub params: Option<HashMap<String, String, RandomState>>,
}

impl RoutingMessage {

    pub fn generate_message(&self, default: &Message) -> Message {
        Message {
            text: self.text.clone().unwrap_or(default.text.clone()),
            starting_module: self.starting_module.clone().unwrap_or(default.starting_module.clone()),
            request_topic: self.request_topic.clone().unwrap_or(default.request_topic.clone()),
            response_topics: self.response_topics.clone().unwrap_or(default.response_topics.clone()),
            sender: self.sender.clone().unwrap_or(default.sender.clone()),
            message_type: self.message_type.clone().unwrap_or(default.message_type.clone()),
            params: default.params.clone(),
        }
    }

}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct RoutingItem {
    pub from_topic: String,
    pub to_topic: String,
    pub message: Option<RoutingMessage>
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Routing {
    pub routing: Vec<RoutingItem>
}

impl Routing {
    pub fn from_file() -> Result<Self, Box<dyn Error>>{
        let contents = fs::read_to_string(ROUTING_FILENAME)?;
        toml::from_str(&contents).map_err(Into::into)
    }
}