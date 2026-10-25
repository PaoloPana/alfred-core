use super::{from_mqtt_topic, parse_status_topic, status_topic, to_mqtt_filter, to_mqtt_topic};

#[test]
fn publish_topic_replaces_dots() {
    assert_eq!(to_mqtt_topic("event.telegram.new_incoming_message").expect("valid topic"), "event/telegram/new_incoming_message");
    assert_eq!(to_mqtt_topic("chat").expect("valid topic"), "chat");
}

#[test]
fn publish_topic_rejects_invalid_topics() {
    for topic in ["", ".", "a..b", ".a", "a.", "a/b", "a.+", "#", "a.#", "a\0b"] {
        assert!(to_mqtt_topic(topic).is_err(), "{topic:?} should be rejected");
    }
}

#[test]
fn filter_matches_topic_and_children() {
    assert_eq!(to_mqtt_filter("a.b").expect("valid topic"), "a/b/#");
    assert_eq!(to_mqtt_filter("chat").expect("valid topic"), "chat/#");
}

#[test]
fn filter_empty_topic_matches_everything() {
    assert_eq!(to_mqtt_filter("").expect("valid topic"), "#");
    assert_eq!(to_mqtt_filter(".").expect("valid topic"), "#");
}

#[test]
fn filter_ignores_one_trailing_dot() {
    assert_eq!(to_mqtt_filter("event.").expect("valid topic"), "event/#");
    assert!(to_mqtt_filter("event..").is_err());
}

#[test]
fn filter_rejects_invalid_topics() {
    for topic in ["a..b", ".a", "a/b", "a.+", "#", "a.#", "+", "a\0"] {
        assert!(to_mqtt_filter(topic).is_err(), "{topic:?} should be rejected");
    }
}

#[test]
fn received_topic_round_trip() {
    for topic in ["a", "a.b", "event.telegram.new_incoming_message", "with-dash.and_underscore.1"] {
        assert_eq!(from_mqtt_topic(&to_mqtt_topic(topic).expect("valid topic")), topic);
    }
}

#[test]
fn received_topic_keeps_non_ascii_and_spaces() {
    assert_eq!(from_mqtt_topic("città/è già"), "città.è già");
}

#[test]
fn status_topics() {
    assert_eq!(status_topic("telegram"), "module/telegram/status");
    assert_eq!(parse_status_topic("module/telegram/status"), Some("telegram"));
    assert_eq!(parse_status_topic("module/telegram-1/status"), Some("telegram-1"));
}

#[test]
fn non_status_topics_are_not_parsed() {
    for topic in ["module/status", "module//status", "module/a/status/x", "module/a/other", "x/a/status", "module/a"] {
        assert_eq!(parse_status_topic(topic), None, "{topic:?}");
    }
}
