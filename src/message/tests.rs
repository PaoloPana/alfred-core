use std::collections::BTreeMap;
use bytes::Bytes;
use serde_derive::Serialize;
use super::{Message, MessageType, PROTOCOL_VERSION};

#[derive(Serialize)]
struct FutureHeader {
    message_type: u8,
    is_final: bool,
    stream_id: String,
    sequence: u32,
    params: BTreeMap<String, String>,
    response_topics: Vec<String>,
    sender: String,
    turn_id: String,
}

#[derive(Serialize)]
struct MinimalHeader {
    message_type: u8,
}

fn header_frame(header: &impl serde::Serialize) -> Bytes {
    let mut frame = vec![PROTOCOL_VERSION];
    rmp_serde::encode::write_named(&mut frame, header).expect("header should encode");
    Bytes::from(frame)
}

#[test]
fn decode_ignores_unknown_header_fields() {
    let header = FutureHeader {
        message_type: MessageType::StreamText.encode(),
        is_final: true,
        stream_id: String::from("stream-1"),
        sequence: 2,
        params: BTreeMap::from([(String::from("key"), String::from("value"))]),
        response_topics: vec![String::from("next.topic")],
        sender: String::from("0123"),
        turn_id: String::from("turn-1"),
    };
    let message = Message::decode(&[header_frame(&header), Bytes::from("data")]).expect("message should decode");
    assert_eq!(message.message_type, MessageType::StreamText);
    assert!(message.is_final);
    assert_eq!(message.stream_id, "stream-1");
    assert_eq!(message.sequence, 2);
    assert_eq!(message.params.get("key").map(String::as_str), Some("value"));
    assert_eq!(message.response_topics.front().map(String::as_str), Some("next.topic"));
    assert_eq!(message.sender, "0123");
    assert_eq!(message.payload, Bytes::from("data"));
}

#[test]
fn decode_uses_defaults_for_missing_header_fields() {
    let header = MinimalHeader { message_type: MessageType::Text.encode() };
    let message = Message::decode(&[header_frame(&header), Bytes::from("data")]).expect("message should decode");
    assert_eq!(message, Message {
        message_type: MessageType::Text,
        payload: Bytes::from("data"),
        ..Message::default()
    });
}
