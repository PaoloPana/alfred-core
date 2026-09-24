use std::collections::BTreeMap;
use alfred_core::config_message::ConfigMessage;
use alfred_core::message::Message;

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
