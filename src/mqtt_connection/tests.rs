use std::collections::HashMap;
use rumqttc::v5::mqttbytes::QoS;
use crate::config::{AlfredConfig, Config};
use crate::error::Error;
use super::{build_settings, client_name, parse_qos, BrokerAddress};

fn address(host: &str, port: u16, tls: bool) -> BrokerAddress {
    BrokerAddress { host: host.to_string(), port, tls }
}

fn config(alfred: AlfredConfig, module_name: Option<&str>) -> Config {
    Config::from_parts(alfred, module_name, HashMap::new())
}

#[test]
fn broker_url_defaults_ports() {
    assert_eq!(BrokerAddress::parse("mqtt://broker.local").expect("valid"), address("broker.local", 1883, false));
    assert_eq!(BrokerAddress::parse("mqtts://broker.local").expect("valid"), address("broker.local", 8883, true));
    assert_eq!(BrokerAddress::parse("mqtt://broker.local/").expect("valid"), address("broker.local", 1883, false));
}

#[test]
fn broker_url_with_port_and_ipv6() {
    assert_eq!(BrokerAddress::parse("mqtt://127.0.0.1:1884").expect("valid"), address("127.0.0.1", 1884, false));
    assert_eq!(BrokerAddress::parse("mqtt://[::1]:1884").expect("valid"), address("::1", 1884, false));
    assert_eq!(BrokerAddress::parse("mqtt://[::1]").expect("valid"), address("::1", 1883, false));
}

#[test]
fn broker_url_rejects_invalid_urls() {
    for url in ["", "127.0.0.1:1883", "tcp://127.0.0.1:1883", "mqtt://", "mqtt://:1883", "mqtt://host:notaport",
        "mqtt://host:99999", "mqtt://user:pass@host", "mqtt://host/path"] {
        assert!(matches!(BrokerAddress::parse(url), Err(Error::InvalidConfig(_))), "{url:?} should be rejected");
    }
}

#[test]
fn qos_values() {
    assert_eq!(parse_qos(0).expect("valid"), QoS::AtMostOnce);
    assert_eq!(parse_qos(1).expect("valid"), QoS::AtLeastOnce);
    assert_eq!(parse_qos(2).expect("valid"), QoS::ExactlyOnce);
    assert!(matches!(parse_qos(3), Err(Error::InvalidConfig(_))));
}

#[test]
fn client_name_with_and_without_instance() {
    assert_eq!(client_name("telegram", None).expect("valid"), "telegram");
    assert_eq!(client_name("telegram", Some("")).expect("valid"), "telegram");
    assert_eq!(client_name("telegram", Some("kitchen")).expect("valid"), "telegram-kitchen");
}

#[test]
fn client_name_rejects_topic_chars() {
    for (name, instance) in [("", None), ("a/b", None), ("a", Some("+")), ("#", None)] {
        assert!(matches!(client_name(name, instance), Err(Error::InvalidConfig(_))), "{name:?} {instance:?}");
    }
}

#[test]
fn named_module_gets_a_persistent_session_and_presence() {
    let settings = build_settings(&config(AlfredConfig::default(), Some("console"))).expect("valid config");
    assert_eq!(settings.options.client_id(), "console");
    assert!(!settings.options.clean_start());
    assert_eq!(settings.options.session_expiry_interval(), Some(3600));
    assert_eq!(settings.status_topic.as_deref(), Some("module/console/status"));
    let will = settings.options.last_will().expect("last will");
    assert_eq!(&will.topic[..], b"module/console/status");
    assert_eq!(&will.message[..], b"offline");
    assert!(will.retain);
}

#[test]
fn instance_id_is_part_of_client_id_and_status_topic() {
    let alfred = AlfredConfig { instance_id: Some(String::from("kitchen")), ..AlfredConfig::default() };
    let settings = build_settings(&config(alfred, Some("console"))).expect("valid config");
    assert_eq!(settings.options.client_id(), "console-kitchen");
    assert_eq!(settings.status_topic.as_deref(), Some("module/console-kitchen/status"));
}

#[test]
fn unnamed_connection_is_throwaway() {
    let first = build_settings(&config(AlfredConfig::default(), None)).expect("valid config");
    let second = build_settings(&config(AlfredConfig::default(), None)).expect("valid config");
    assert!(first.options.clean_start());
    assert_eq!(first.options.session_expiry_interval(), Some(0));
    assert!(first.options.last_will().is_none());
    assert!(first.status_topic.is_none());
    assert_ne!(first.options.client_id(), second.options.client_id());
}

#[test]
fn settings_reject_invalid_config() {
    let bad_qos = AlfredConfig { qos: 7, ..AlfredConfig::default() };
    let bad_url = AlfredConfig { broker_url: String::from("tcp://x"), ..AlfredConfig::default() };
    let ca_without_tls = AlfredConfig { ca_file: Some(String::from("ca.pem")), ..AlfredConfig::default() };
    let missing_ca = AlfredConfig {
        broker_url: String::from("mqtts://localhost"),
        ca_file: Some(String::from("/nonexistent/ca.pem")),
        ..AlfredConfig::default()
    };
    for alfred in [bad_qos, bad_url, ca_without_tls, missing_ca] {
        assert!(matches!(build_settings(&config(alfred, Some("m"))), Err(Error::InvalidConfig(_))));
    }
}

#[test]
fn credentials_are_set() {
    let alfred = AlfredConfig {
        username: Some(String::from("alfred")),
        password: Some(String::from("secret")),
        ..AlfredConfig::default()
    };
    let settings = build_settings(&config(alfred, Some("m"))).expect("valid config");
    let login = settings.options.credentials().expect("credentials");
    assert_eq!((login.username.as_str(), login.password.as_str()), ("alfred", "secret"));
}

#[test]
fn debug_output_hides_the_password() {
    let alfred = AlfredConfig { password: Some(String::from("secret")), ..AlfredConfig::default() };
    let output = format!("{:?}", config(alfred, None));
    assert!(!output.contains("secret"));
}
