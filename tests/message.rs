use alfred_core::bytes::Bytes;
use alfred_core::error::{Error, MessageEncodingError};
use alfred_core::message::{Message, MessageType, PROTOCOL_VERSION};
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

fn round_trip(message: &Message) -> Message {
    let frames = message.encode().expect("message should encode");
    Message::decode(&frames).expect("encoded message should decode")
}

#[test]
fn encode_decode_message_type_round_trip() {
    for message_type in &ALL_TYPES {
        let encoded = message_type.encode();
        let decoded = MessageType::decode(encoded).expect("encoded value should decode");
        assert_eq!(*message_type, decoded);
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
    assert_ne!(MessageType::StreamText.encode(), MessageType::Text.encode());
    assert_ne!(MessageType::StreamAudio.encode(), MessageType::Audio.encode());
    assert_ne!(MessageType::StreamPhoto.encode(), MessageType::Photo.encode());
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
                payload: Bytes::from("data"),
                response_topics: LinkedList::from([String::from("a.b"), String::from("c.d")]),
                sender: String::from("0123"),
                params: BTreeMap::from([(String::from("par1"), String::from("val1"))]),
            };
            assert_eq!(message, round_trip(&message));
        }
    }
}

#[test]
fn binary_payload_round_trip() {
    let payload = (0..=u8::MAX).chain([0x00, 0xFF, 0x00]).collect::<Vec<u8>>();
    let message = Message {
        message_type: MessageType::StreamAudio,
        payload: Bytes::from(payload),
        ..Message::default()
    };
    let decoded = round_trip(&message);
    assert_eq!(message, decoded);
    assert!(matches!(decoded.text(), Err(Error::PayloadNotText)));
    assert_eq!(decoded.payload_description(), "<259 bytes>");
}

#[test]
fn fields_containing_separator_and_many_params_round_trip() {
    let message = Message {
        sender: String::from("se\0nder"),
        stream_id: String::from("stream\0id"),
        response_topics: LinkedList::from([String::from("a\0b")]),
        params: (0..300).map(|index| (format!("key\0{index}"), format!("value\0{index}"))).collect(),
        payload: Bytes::from("te\0xt"),
        ..Message::default()
    };
    assert_eq!(message, round_trip(&message));
}

#[test]
fn text_returns_utf8_payload() {
    let message = Message { payload: Bytes::from("città"), ..Message::default() };
    assert_eq!(message.text().expect("payload should be text"), "città");
    assert_eq!(message.payload_description(), "città");
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
    assert_eq!(response.payload, Bytes::from("chunk"));
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

#[test]
fn decode_rejects_other_protocol_version() {
    let [header, payload] = Message::default().encode().expect("message should encode");
    let mut header = header.to_vec();
    header[0] = PROTOCOL_VERSION + 1;
    let result = Message::decode(&[Bytes::from(header), payload]);
    assert!(matches!(result, Err(MessageEncodingError::ProtocolVersion(version)) if version == PROTOCOL_VERSION + 1));
}

#[test]
fn decode_rejects_legacy_string_messages() {
    let legacy_text = Bytes::from(format!("{}{}{}{}0123\0\00\0data", 0x01 as char, 0x00 as char, 0x00 as char, 0x00 as char));
    let legacy_stream = Bytes::from(format!("{}{}{}{}0123\0\00\0data", 0x81 as char, 0x00 as char, 0x00 as char, 0x00 as char));
    let legacy_module_info = Bytes::from(format!("{}{}{}{}0123\0\00\0data", 0xFF as char, 0x00 as char, 0x00 as char, 0x00 as char));
    let unreleased_v4 = Bytes::from(format!("{}{}{}{}{}0123\0\00\0data", 0x04 as char, 0x01 as char, 0x00 as char, 0x00 as char, 0x00 as char));
    for legacy in [legacy_text, legacy_stream, legacy_module_info, unreleased_v4] {
        assert!(matches!(Message::decode(&[legacy]), Err(MessageEncodingError::ProtocolVersion(_))));
    }
}

#[test]
fn decode_rejects_legacy_photo_message_with_same_first_byte() {
    let legacy_photo = Bytes::from(format!("{}{}{}{}0123\0\00\0data", 0x03 as char, 0x00 as char, 0x00 as char, 0x00 as char));
    assert_eq!(legacy_photo.first(), Some(&PROTOCOL_VERSION));
    assert!(matches!(Message::decode(&[legacy_photo]), Err(MessageEncodingError::FieldNotFound(_))));
}

#[test]
fn decode_returns_error_on_missing_frames() {
    let [header, _] = Message::default().encode().expect("message should encode");
    assert!(matches!(Message::decode(&[]), Err(MessageEncodingError::FieldNotFound(_))));
    assert!(matches!(Message::decode(&[Bytes::new()]), Err(MessageEncodingError::FieldNotFound(_))));
    assert!(matches!(Message::decode(&[header]), Err(MessageEncodingError::FieldNotFound(_))));
}

#[test]
fn decode_returns_error_on_malformed_header() {
    let [header, payload] = Message::default().encode().expect("message should encode");
    for len in 1..header.len() {
        let truncated = header.slice(0..len);
        assert!(matches!(Message::decode(&[truncated, payload.clone()]), Err(MessageEncodingError::Header(_))));
    }
}

#[test]
fn decode_returns_error_on_unknown_message_type() {
    let message = Message { message_type: MessageType::Text, ..Message::default() };
    let [header, payload] = message.encode().expect("message should encode");
    let mut header = header.to_vec();
    let type_position = header.iter().position(|byte| *byte == MessageType::Text.encode())
        .expect("header should contain the message type");
    header[type_position] = 0x42;
    assert!(matches!(Message::decode(&[Bytes::from(header), payload]), Err(MessageEncodingError::MessageType(_))));
}
