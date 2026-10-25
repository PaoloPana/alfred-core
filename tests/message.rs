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

/// Text message with id `i`, no other metadata and payload `hi` (see docs/protocol.md)
const GOLDEN: [u8; 91] = [
    0x04, 0x00, 0x00, 0x00, 0x54, 0x88, 0xA2, 0x69, 0x64, 0xA1, 0x69, 0xAC,
    0x6D, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x5F, 0x74, 0x79, 0x70, 0x65,
    0x01, 0xA8, 0x69, 0x73, 0x5F, 0x66, 0x69, 0x6E, 0x61, 0x6C, 0xC2, 0xA9,
    0x73, 0x74, 0x72, 0x65, 0x61, 0x6D, 0x5F, 0x69, 0x64, 0xA0, 0xA8, 0x73,
    0x65, 0x71, 0x75, 0x65, 0x6E, 0x63, 0x65, 0x00, 0xA6, 0x70, 0x61, 0x72,
    0x61, 0x6D, 0x73, 0x80, 0xAF, 0x72, 0x65, 0x73, 0x70, 0x6F, 0x6E, 0x73,
    0x65, 0x5F, 0x74, 0x6F, 0x70, 0x69, 0x63, 0x73, 0x90, 0xA6, 0x73, 0x65,
    0x6E, 0x64, 0x65, 0x72, 0xA0, 0x68, 0x69,
];

fn round_trip(message: &Message) -> Message {
    let bytes = message.encode().expect("message should encode");
    Message::decode(&bytes).expect("encoded message should decode")
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
                id: String::from("0b0d7d2e-6c1c-4a4e-9d51-0f2b2c1e7a10"),
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
    let mut bytes = Message::default().encode().expect("message should encode").to_vec();
    bytes[0] = PROTOCOL_VERSION + 1;
    let result = Message::decode(&Bytes::from(bytes));
    assert!(matches!(result, Err(MessageEncodingError::ProtocolVersion(version)) if version == PROTOCOL_VERSION + 1));
}

#[test]
fn decode_rejects_legacy_zeromq_and_string_messages() {
    // version 3 (ZeroMQ, three frames): the first frame started with its protocol version
    let zeromq_v3 = Bytes::from_static(&[0x03, 0x88, 0xAC, b'm']);
    let legacy_text = Bytes::from(format!("{}{}{}{}0123\0\00\0data", 0x01 as char, 0x00 as char, 0x00 as char, 0x00 as char));
    let legacy_stream = Bytes::from(format!("{}{}{}{}0123\0\00\0data", 0x81 as char, 0x00 as char, 0x00 as char, 0x00 as char));
    let legacy_module_info = Bytes::from(format!("{}{}{}{}0123\0\00\0data", 0xFF as char, 0x00 as char, 0x00 as char, 0x00 as char));
    for legacy in [zeromq_v3, legacy_text, legacy_stream, legacy_module_info] {
        assert!(matches!(Message::decode(&legacy), Err(MessageEncodingError::ProtocolVersion(_))));
    }
}

#[test]
fn decode_returns_error_on_empty_or_short_data() {
    assert!(matches!(Message::decode(&Bytes::new()), Err(MessageEncodingError::FieldNotFound(_))));
    for len in 1..=4 {
        let short = Bytes::from(vec![PROTOCOL_VERSION; len]);
        assert!(matches!(Message::decode(&short), Err(MessageEncodingError::FieldNotFound(_))), "length {len}");
    }
}

#[test]
fn decode_returns_error_on_malformed_header() {
    let bytes = Message::default().encode().expect("message should encode");
    // the payload is empty, so every truncation cuts the header (or its length field)
    for len in 5..bytes.len() {
        assert!(matches!(Message::decode(&bytes.slice(0..len)), Err(MessageEncodingError::Header(_))), "length {len}");
    }
}

#[test]
fn decode_returns_error_on_huge_header_length() {
    let mut bytes = vec![PROTOCOL_VERSION];
    bytes.extend_from_slice(&u32::MAX.to_be_bytes());
    bytes.extend_from_slice(b"short");
    assert!(matches!(Message::decode(&Bytes::from(bytes)), Err(MessageEncodingError::Header(_))));
}

#[test]
fn decode_returns_error_on_unknown_message_type() {
    let message = Message { message_type: MessageType::Text, ..Message::default() };
    let mut bytes = message.encode().expect("message should encode").to_vec();
    let type_position = bytes.iter().skip(5).position(|byte| *byte == MessageType::Text.encode())
        .expect("header should contain the message type") + 5;
    bytes[type_position] = 0x42;
    assert!(matches!(Message::decode(&Bytes::from(bytes)), Err(MessageEncodingError::MessageType(_))));
}

#[test]
fn encode_with_id_overrides_the_message_id() {
    let message = Message { id: String::from("original"), ..Message::default() };
    let decoded = Message::decode(&message.encode_with_id("other").expect("message should encode")).expect("should decode");
    assert_eq!(decoded.id, "other");
    assert_eq!(round_trip(&message).id, "original");
}

#[test]
fn wire_layout_is_version_length_header_payload() {
    let message = Message {
        id: String::from("i"),
        message_type: MessageType::Text,
        payload: Bytes::from("hi"),
        ..Message::default()
    };
    let bytes = message.encode().expect("message should encode");
    assert_eq!(bytes[0], PROTOCOL_VERSION);
    let header_length = u32::from_be_bytes(bytes[1..5].try_into().expect("4 bytes")) as usize;
    assert_eq!(bytes.len(), 5 + header_length + 2);
    assert_eq!(&bytes[5 + header_length..], b"hi");
    // golden bytes, also documented in docs/protocol.md
    assert_eq!(&bytes[..], &GOLDEN[..]);
    assert_eq!(Message::decode(&Bytes::from_static(&GOLDEN)).expect("golden bytes should decode"), message);
}
