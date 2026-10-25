//! Integration tests against a real MQTT 5 broker (e.g. Mosquitto, see `mosquitto.conf`).
//! They run only if `ALFRED_TEST_BROKER_URL` is set (e.g. `mqtt://127.0.0.1:1883`), as in CI.
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;
use alfred_core::bytes::Bytes;
use alfred_core::config::{AlfredConfig, Config};
use alfred_core::connection::{Connection, MODULE_INFO_TOPIC_REQUEST, MODULE_INFO_TOPIC_RESPONSE};
use alfred_core::message::{Message, MessageType};
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, MqttOptions};
use tokio::time::timeout;
use uuid::Uuid;

const TIMEOUT: Duration = Duration::from_secs(10);

fn broker_url() -> Option<String> {
    let url = std::env::var("ALFRED_TEST_BROKER_URL").ok();
    if url.is_none() {
        eprintln!("ALFRED_TEST_BROKER_URL is not set: skipping the test");
    }
    url
}

fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::new_v4().simple())
}

async fn connect(broker_url: &str, module_name: &str) -> Connection {
    let alfred = AlfredConfig {
        broker_url: broker_url.to_string(),
        session_expiry_secs: 60,
        ..AlfredConfig::default()
    };
    let config = Config::from_parts(alfred, Some(module_name), HashMap::new());
    Connection::new(&config).await.expect("connection to the broker")
}

/// Next message, ignoring the module info requests that the other tests broadcast to every module
async fn receive(connection: &Connection) -> (String, Message) {
    loop {
        let received = timeout(TIMEOUT, connection.receive_all()).await.expect("message within the timeout").expect("message");
        if received.0 != MODULE_INFO_TOPIC_REQUEST {
            return received;
        }
    }
}

fn text(payload: &str) -> Message {
    Message { message_type: MessageType::Text, payload: Bytes::from(payload.to_string()), ..Message::default() }
}

#[tokio::test]
async fn two_modules_exchange_messages() {
    let Some(url) = broker_url() else { return };
    let topic = format!("it.{}.request", unique("a"));
    let reply_topic = format!("it.{}.reply", unique("a"));
    let mut client = connect(&url, &unique("client")).await;
    let mut server = connect(&url, &unique("server")).await;
    server.listen(&topic).await.expect("listen");
    client.listen(&reply_topic).await.expect("listen");

    let request = Message {
        params: BTreeMap::from([(String::from("key"), String::from("value"))]),
        response_topics: [reply_topic.clone()].into(),
        sender: String::from("client"),
        ..text("ping")
    };
    client.send(&topic, &request).await.expect("send");

    let (received_topic, received) = receive(&server).await;
    assert_eq!(received_topic, topic);
    assert_eq!(received.text().expect("text"), "ping");
    assert_eq!(received.params, request.params);
    assert_ne!(received.id, "");

    let (response_topic, response) = received.reply("pong", MessageType::Text).expect("reply");
    assert_eq!(response_topic, reply_topic);
    server.send(&response_topic, &response).await.expect("send");
    let (received_topic, received) = receive(&client).await;
    assert_eq!(received_topic, reply_topic);
    assert_eq!(received.text().expect("text"), "pong");
    assert_eq!(received.sender, "client");
}

#[tokio::test]
async fn every_publication_has_a_new_id() {
    let Some(url) = broker_url() else { return };
    let topic = format!("it.{}.ids", unique("a"));
    let mut receiver = connect(&url, &unique("receiver")).await;
    let sender = connect(&url, &unique("sender")).await;
    receiver.listen(&topic).await.expect("listen");
    let message = text("same message");
    sender.send(&topic, &message).await.expect("send");
    sender.send(&topic, &message).await.expect("send");
    let (_, first) = receive(&receiver).await;
    let (_, second) = receive(&receiver).await;
    assert_ne!(first.id, "");
    assert_ne!(second.id, "");
    assert_ne!(first.id, second.id);
    assert_eq!(first.payload, second.payload);
}

#[tokio::test]
async fn subscriptions_match_whole_segments_only() {
    let Some(url) = broker_url() else { return };
    let base = format!("it.{}", unique("seg"));
    let mut receiver = connect(&url, &unique("receiver")).await;
    let sender = connect(&url, &unique("sender")).await;
    receiver.listen(&format!("{base}.console")).await.expect("listen");
    for suffix in ["console", "consolex", "console.sub", "other", "console.end"] {
        sender.send(&format!("{base}.{suffix}"), &text(suffix)).await.expect("send");
    }
    let mut topics_seen = Vec::new();
    while topics_seen.last().map(String::as_str) != Some("console.end") {
        let (topic, message) = receive(&receiver).await;
        assert_eq!(topic, format!("{base}.{}", message.text().expect("text")));
        topics_seen.push(message.text().expect("text").to_string());
    }
    assert_eq!(topics_seen, ["console", "console.sub", "console.end"]);
}

#[tokio::test]
async fn empty_topic_listens_to_everything() {
    let Some(url) = broker_url() else { return };
    let topic = format!("it.{}.anything", unique("all"));
    let mut receiver = connect(&url, &unique("receiver")).await;
    let sender = connect(&url, &unique("sender")).await;
    receiver.listen("").await.expect("listen");
    sender.send(&topic, &text("x")).await.expect("send");
    loop {
        let (received_topic, _) = receive(&receiver).await;
        if received_topic == topic { break; }
    }
}

async fn persistent_session_receives_messages_sent_while_offline(clean_disconnect: bool) {
    let Some(url) = broker_url() else { return };
    let topic = format!("it.{}.offline", unique("p"));
    let name = unique("persistent");
    let status_topic = format!("module.{name}.status");

    let mut observer = connect(&url, &unique("observer")).await;
    observer.listen(&status_topic).await.expect("listen");
    let mut module = connect(&url, &name).await;
    module.listen(&topic).await.expect("listen");
    let (received_topic, status) = receive(&observer).await;
    assert_eq!((received_topic.as_str(), status.text().expect("text")), (status_topic.as_str(), "online"));
    assert_eq!(status.params.get("module"), Some(&name));

    if clean_disconnect {
        module.disconnect().await.expect("disconnect");
    }
    drop(module);
    let (_, status) = receive(&observer).await;
    assert_eq!(status.text().expect("text"), "offline");

    let sender = connect(&url, &unique("sender")).await;
    for i in 0..3 {
        sender.send(&topic, &text(&format!("while-offline-{i}"))).await.expect("send");
    }

    // the module restarts and does not subscribe again: the session still has its subscription
    let module = connect(&url, &name).await;
    for i in 0..3 {
        let (received_topic, message) = receive(&module).await;
        assert_eq!(received_topic, topic);
        assert_eq!(message.text().expect("text"), format!("while-offline-{i}"));
    }
    let (_, status) = receive(&observer).await;
    assert_eq!(status.text().expect("text"), "online");
}

#[tokio::test]
async fn persistent_session_after_clean_disconnect() {
    persistent_session_receives_messages_sent_while_offline(true).await;
}

#[tokio::test]
async fn persistent_session_after_dropped_connection() {
    persistent_session_receives_messages_sent_while_offline(false).await;
}

#[tokio::test]
async fn duplicate_delivery_is_dropped() {
    let Some(url) = broker_url() else { return };
    let topic = unique("dup");
    let mut receiver = connect(&url, &unique("receiver")).await;
    receiver.listen(&format!("it.{topic}")).await.expect("listen");

    // a raw client publishes the very same bytes (so the same message id) twice, as a QoS 1 retransmission would
    let (host, port) = url.trim_start_matches("mqtt://").split_once(':').expect("host:port");
    let mut options = MqttOptions::new(unique("raw"), host, port.parse::<u16>().expect("port"));
    options.set_keep_alive(Duration::from_secs(5));
    let (raw, mut event_loop) = AsyncClient::new(options, 10);
    let poller = tokio::spawn(async move { while event_loop.poll().await.is_ok() {} });
    let mqtt_topic = format!("it/{topic}");
    let duplicated = Message { id: Uuid::new_v4().to_string(), ..text("duplicated") }.encode().expect("encode");
    let sentinel = Message { id: Uuid::new_v4().to_string(), ..text("sentinel") }.encode().expect("encode");
    for payload in [duplicated.clone(), duplicated, sentinel] {
        raw.publish(mqtt_topic.clone(), QoS::AtLeastOnce, false, payload).await.expect("publish");
    }

    let (_, first) = receive(&receiver).await;
    let (_, second) = receive(&receiver).await;
    assert_eq!((first.text().expect("text"), second.text().expect("text")), ("duplicated", "sentinel"));
    poller.abort();
}

#[tokio::test]
async fn undecodable_messages_are_discarded() {
    let Some(url) = broker_url() else { return };
    let topic = unique("garbage");
    let mut receiver = connect(&url, &unique("receiver")).await;
    receiver.listen(&format!("it.{topic}")).await.expect("listen");
    let sender = connect(&url, &unique("sender")).await;
    let module_name = unique("m");
    // a foreign client (e.g. Home Assistant) publishes something that is not an Alfred message
    let (host, port) = url.trim_start_matches("mqtt://").split_once(':').expect("host:port");
    let (raw, mut event_loop) = AsyncClient::new(MqttOptions::new(unique("raw"), host, port.parse::<u16>().expect("port")), 10);
    let poller = tokio::spawn(async move { while event_loop.poll().await.is_ok() {} });
    raw.publish(format!("it/{topic}"), QoS::AtLeastOnce, false, "not an alfred message").await.expect("publish");
    sender.send(&format!("it.{topic}"), &text("valid")).await.expect("send");
    let result = timeout(TIMEOUT, receiver.receive(&module_name, &BTreeMap::new())).await.expect("timeout").expect("message");
    assert_eq!(result.1.text().expect("text"), "valid");
    poller.abort();
}

#[tokio::test]
async fn module_info_request_is_answered() {
    let Some(url) = broker_url() else { return };
    let name = unique("info");
    let mut requester = connect(&url, &unique("requester")).await;
    requester.listen(MODULE_INFO_TOPIC_RESPONSE).await.expect("listen");
    let module = connect(&url, &name).await;
    let capabilities = BTreeMap::from([(String::from("event"), String::from("x"))]);
    let responder = {
        let (name, capabilities) = (name.clone(), capabilities.clone());
        tokio::spawn(async move { let _ = module.receive(&name, &capabilities).await; })
    };
    requester.send(MODULE_INFO_TOPIC_REQUEST, &Message::default()).await.expect("send");
    loop {
        let (topic, message) = receive(&requester).await;
        assert_eq!(topic, MODULE_INFO_TOPIC_RESPONSE);
        if message.text().expect("text") == name {
            assert_eq!(message.params, capabilities);
            break;
        }
    }
    responder.abort();
}
