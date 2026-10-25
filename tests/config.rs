//! The configuration is read from `ALFRED_CONFIG` and the environment, which are process-wide:
//! everything runs in a single test, one scenario after the other.
use std::fs;
use std::panic::catch_unwind;
use alfred_core::config::Config;

const ENV_VARIABLES: [&str; 11] = [
    "ALFRED_CONFIG", "ALFRED_BROKER_URL", "ALFRED_USERNAME", "ALFRED_PASSWORD", "ALFRED_CA_FILE", "ALFRED_QOS",
    "ALFRED_SESSION_EXPIRY_SECS", "ALFRED_INSTANCE_ID", "ALFRED_URL", "ALFRED_PUB_PORT", "ALFRED_SUB_PORT",
];

fn read_with(file_contents: &str, env: &[(&str, &str)]) -> Result<Config, String> {
    for variable in ENV_VARIABLES {
        std::env::remove_var(variable);
    }
    let path = std::env::temp_dir().join(format!("alfred-config-test-{}.toml", std::process::id()));
    fs::write(&path, file_contents).expect("temporary config file");
    std::env::set_var("ALFRED_CONFIG", &path);
    for (key, value) in env {
        std::env::set_var(key, value);
    }
    let result = catch_unwind(|| Config::read(Some("test")));
    fs::remove_file(path).expect("temporary config file");
    result.map_err(|panic| {
        panic.downcast_ref::<String>().cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(ToString::to_string))
            .unwrap_or_default()
    })
}

#[test]
fn config_scenarios() {
    // an empty [alfred] section uses the defaults
    let config = read_with("[alfred]\n[test]\n", &[]).expect("valid config");
    assert_eq!(config.alfred.broker_url, "mqtt://127.0.0.1:1883");
    assert_eq!((config.alfred.qos, config.alfred.session_expiry_secs), (1, 3600));
    assert!(config.alfred.username.is_none() && config.alfred.password.is_none() && config.alfred.ca_file.is_none());
    assert!(config.alfred.instance_id.is_none());
    assert_eq!(config.get_module_name(), Some("test"));

    // values from file
    let config = read_with(
        "[alfred]\nbroker_url = \"mqtts://broker.lan:8884\"\nusername = \"u\"\nqos = 2\nsession_expiry_secs = 60\ninstance_id = \"i\"\nmodules = [\"routing\"]\n",
        &[],
    ).expect("valid config");
    assert_eq!(config.alfred.broker_url, "mqtts://broker.lan:8884");
    assert_eq!(config.alfred.username.as_deref(), Some("u"));
    assert_eq!((config.alfred.qos, config.alfred.session_expiry_secs), (2, 60));
    assert_eq!(config.alfred.instance_id.as_deref(), Some("i"));
    assert_eq!(config.alfred.modules, ["routing"]);

    // the environment wins over the file, so secrets do not have to be in config.toml
    let config = read_with(
        "[alfred]\nbroker_url = \"mqtt://from-file\"\nqos = 0\n",
        &[("ALFRED_BROKER_URL", "mqtt://from-env:1999"), ("ALFRED_PASSWORD", "s3cret"), ("ALFRED_USERNAME", "alfred"), ("ALFRED_QOS", "2")],
    ).expect("valid config");
    assert_eq!(config.alfred.broker_url, "mqtt://from-env:1999");
    assert_eq!(config.alfred.password.as_deref(), Some("s3cret"));
    assert_eq!(config.alfred.username.as_deref(), Some("alfred"));
    assert_eq!(config.alfred.qos, 2);
    assert!(!format!("{config:?}").contains("s3cret"));

    // old ZeroMQ keys fail with a migration message, in the file and in the environment
    for file in ["[alfred]\nurl = \"tcp://127.0.0.1\"\npub_port = 5678\nsub_port = 1234\n", "[alfred]\nbroker_url = \"mqtt://x\"\npub_port = 5678\n"] {
        let message = read_with(file, &[]).expect_err("old keys must be rejected");
        assert!(message.contains("broker_url") && message.contains("docs/broker.md"), "{message}");
    }
    let message = read_with("[alfred]\n", &[("ALFRED_PUB_PORT", "5678")]).expect_err("old variables must be rejected");
    assert!(message.contains("ALFRED_BROKER_URL"), "{message}");
}
