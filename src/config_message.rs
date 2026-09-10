use std::collections::{BTreeMap, LinkedList};
use serde_derive::Deserialize;
use crate::message::{Message, MessageType};

#[derive(Deserialize, Clone, Debug)]
pub struct ConfigMessage {
    pub text: Option<String>,
    pub response_topics: Option<LinkedList<String>>,
    pub sender: Option<String>,
    pub message_type: Option<MessageType>,
    pub params: Option<BTreeMap<String, String>>,
}

impl ConfigMessage {
    pub fn generate_message(&self, default: &Message) -> Message {
        let mut params = default.params.clone();
        if let Some(config_params) = &self.params {
            params.extend(config_params.clone());
        }
        Message {
            text: self.text.clone().unwrap_or_else(|| default.text.clone()),
            response_topics: self.response_topics.clone().unwrap_or_else(|| default.response_topics.clone()),
            sender: self.sender.clone().unwrap_or_else(|| default.sender.clone()),
            message_type: self.message_type.clone().unwrap_or_else(|| default.message_type.clone()),
            is_final: default.is_final,
            stream_id: default.stream_id.clone(),
            sequence: default.sequence,
            params,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use crate::message::Message;
    use super::ConfigMessage;

    #[test]
    fn generate_message_merges_params() {
        let config_message = ConfigMessage {
            text: None,
            response_topics: None,
            sender: None,
            message_type: None,
            params: Some(BTreeMap::from([
                (String::from("stream"), String::from("true")),
                (String::from("shared"), String::from("config")),
            ])),
        };
        let incoming = Message {
            params: BTreeMap::from([
                (String::from("request"), String::from("hi")),
                (String::from("shared"), String::from("incoming")),
            ]),
            ..Message::default()
        };
        let generated = config_message.generate_message(&incoming);
        assert_eq!(generated.params, BTreeMap::from([
            (String::from("request"), String::from("hi")),
            (String::from("shared"), String::from("config")),
            (String::from("stream"), String::from("true")),
        ]));
    }
}
