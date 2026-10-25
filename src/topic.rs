//! Mapping between Alfred topics (dot separated, e.g. `event.telegram.new_message`) and MQTT
//! topics (slash separated). This is the only place where the conversion happens.
//!
//! Subscriptions keep a "prefix" semantic, but only at segment boundaries: listening to `a.b`
//! receives `a.b` and `a.b.c`, never `a.bc`. An empty topic listens to everything.
use crate::error::Error;

const ALFRED_SEPARATOR: char = '.';
const MQTT_SEPARATOR: char = '/';
const MQTT_MULTI_LEVEL: char = '#';
const MQTT_SINGLE_LEVEL: char = '+';

pub const STATUS_TOPIC_ROOT: &str = "module";
pub const STATUS_TOPIC_LEAF: &str = "status";

fn validate(topic: &str) -> Result<(), Error> {
    let forbidden = [MQTT_SEPARATOR, MQTT_MULTI_LEVEL, MQTT_SINGLE_LEVEL, '\0'];
    if topic.contains(forbidden) {
        return Err(Error::InvalidTopic(topic.to_string()));
    }
    Ok(())
}

/// Converts the topic of a message to publish: `a.b` becomes `a/b`.
/// Empty topics, empty segments and the chars `/`, `+`, `#` and `\0` are rejected.
pub fn to_mqtt_topic(topic: &str) -> Result<String, Error> {
    validate(topic)?;
    if topic.is_empty() || topic.split(ALFRED_SEPARATOR).any(str::is_empty) {
        return Err(Error::InvalidTopic(topic.to_string()));
    }
    Ok(topic.replace(ALFRED_SEPARATOR, "/"))
}

/// Converts a topic to listen to into an MQTT topic filter: `a.b` becomes `a/b/#` (which also
/// matches `a/b` itself), an empty topic becomes `#`. A trailing `.` is ignored.
pub fn to_mqtt_filter(topic: &str) -> Result<String, Error> {
    validate(topic)?;
    let topic = topic.strip_suffix(ALFRED_SEPARATOR).unwrap_or(topic);
    if topic.is_empty() {
        return Ok(MQTT_MULTI_LEVEL.to_string());
    }
    if topic.split(ALFRED_SEPARATOR).any(str::is_empty) {
        return Err(Error::InvalidTopic(topic.to_string()));
    }
    Ok(format!("{}/{MQTT_MULTI_LEVEL}", topic.replace(ALFRED_SEPARATOR, "/")))
}

/// Converts the topic of a received MQTT message: `a/b` becomes `a.b`.
pub fn from_mqtt_topic(topic: &str) -> String {
    topic.replace(MQTT_SEPARATOR, ".")
}

/// MQTT topic of the presence (online/offline) message of a client.
pub fn status_topic(client_name: &str) -> String {
    format!("{STATUS_TOPIC_ROOT}/{client_name}/{STATUS_TOPIC_LEAF}")
}

/// Returns the client name if `mqtt_topic` is a presence topic (`module/<name>/status`).
pub fn parse_status_topic(mqtt_topic: &str) -> Option<&str> {
    let mut segments = mqtt_topic.split(MQTT_SEPARATOR);
    match (segments.next(), segments.next(), segments.next(), segments.next()) {
        (Some(STATUS_TOPIC_ROOT), Some(name), Some(STATUS_TOPIC_LEAF), None) if !name.is_empty() => Some(name),
        _ => None
    }
}

#[cfg(test)]
mod tests;
