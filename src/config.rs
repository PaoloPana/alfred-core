use std::collections::HashMap;
use std::fmt;
use serde_derive::Deserialize;
use std::fs;
use std::path::Path;
use toml;
use envconfig::Envconfig;
use toml::{Table, Value};

pub const CONFIG_FILENAME: &str = "config.toml";
pub const DEFAULT_BROKER_URL: &str = "mqtt://127.0.0.1:1883";
pub const DEFAULT_QOS: u8 = 1;
pub const DEFAULT_SESSION_EXPIRY_SECS: u32 = 3600;
const DEFAULT_TMP_DIR: &str = "/tmp";

const MIGRATION_MESSAGE: &str = "alfred-core 0.5 uses an MQTT 5 broker instead of the ZeroMQ daemon: \
the keys `url`, `pub_port` and `sub_port` (and the env variables ALFRED_URL, ALFRED_PUB_PORT, ALFRED_SUB_PORT) \
are not supported anymore. Set `broker_url` (e.g. broker_url = \"mqtt://127.0.0.1:1883\") in the [alfred] \
section instead (or ALFRED_BROKER_URL), remove \"daemon\" from `modules` and start an MQTT broker such as Mosquitto \
(see docs/broker.md)";

#[derive(Deserialize)]
pub struct Config {
    pub alfred: AlfredConfig,
    module: HashMap<String, String>,
    #[serde(skip)]
    module_name: Option<String>
}

impl Config {
    /// # Panics
    /// Panics if the configuration cannot be read, or if it still contains the `ZeroMQ` keys
    /// (`url`, `pub_port`, `sub_port`), with a message explaining how to migrate.
    pub fn read(module_name: Option<&str>) -> Self {
        let alfred = Self::read_alfred_config();
        let module = module_name.map_or_else(HashMap::new, Self::read_module_config);
        Self { alfred, module, module_name: module_name.map(str::to_string) }
    }

    /// Builds a configuration without reading `config.toml` and the environment.
    pub fn from_parts(alfred: AlfredConfig, module_name: Option<&str>, module: HashMap<String, String>) -> Self {
        Self { alfred, module, module_name: module_name.map(str::to_string) }
    }

    /// Sets the module name if the configuration has none (it identifies the module on the broker).
    #[must_use]
    pub fn with_default_module_name(mut self, module_name: &str) -> Self {
        if self.module_name.is_none() {
            self.module_name = Some(module_name.to_string());
        }
        self
    }

    fn read_module_config(module_name: &str) -> HashMap<String, String> {
        let contents = fs::read_to_string(CONFIG_FILENAME).expect("Could not read file");
        let table: Table = contents.parse().expect("Could not parse toml file");
        table.get(&module_name.to_string())
            .and_then(Value::as_table)
            .map_or_else(HashMap::new, |module_config| module_config
                .iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                .collect())
    }

    fn read_alfred_config() -> AlfredConfig {
        let from_env = EnvConfig::from_env();
        let from_file_config = FromFileConfig::read();
        let file = from_file_config.alfred;
        let env = from_env.alfred;
        assert!(
            [env.legacy_url.is_some(), env.legacy_pub_port.is_some(), env.legacy_sub_port.is_some(),
                file.url.is_some(), file.pub_port.is_some(), file.sub_port.is_some()].iter().all(|found| !found),
            "{MIGRATION_MESSAGE}"
        );
        AlfredConfig {
            broker_url: env.broker_url.or(file.broker_url).unwrap_or_else(|| DEFAULT_BROKER_URL.to_string()),
            username: env.username.or(file.username),
            password: env.password.or(file.password),
            ca_file: env.ca_file.or(file.ca_file),
            qos: env.qos.or(file.qos).unwrap_or(DEFAULT_QOS),
            session_expiry_secs: env.session_expiry_secs.or(file.session_expiry_secs).unwrap_or(DEFAULT_SESSION_EXPIRY_SECS),
            instance_id: env.instance_id.or(file.instance_id),
            tmp_dir: env.tmp_dir.or(file.tmp_dir).unwrap_or_else(|| DEFAULT_TMP_DIR.to_string()),
            modules: file.modules,
        }
    }

    pub fn get_module_value(&self, key: &str) -> Option<String> {
        self.module.get(key).cloned()
    }

    /// Name of the module this configuration was read for, if any.
    pub fn get_module_name(&self) -> Option<&str> {
        self.module_name.as_deref()
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("alfred", &self.alfred)
            .field("module", &self.module)
            .field("module_name", &self.module_name)
            .finish()
    }
}

#[derive(Deserialize, Clone)]
pub struct AlfredConfig {
    /// `mqtt://host[:port]` (plain TCP, default port 1883) or `mqtts://host[:port]` (TLS, default port 8883)
    pub broker_url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    /// PEM file with the CA certificate(s) to trust instead of the system ones (`mqtts://` only)
    pub ca_file: Option<String>,
    /// MQTT `QoS` used to publish and subscribe: 0 (at most once), 1 (at least once) or 2 (exactly once)
    pub qos: u8,
    /// Seconds the broker keeps the session (subscriptions and queued messages) of a disconnected module
    pub session_expiry_secs: u32,
    /// Distinguishes several instances of the same module connected to the same broker
    pub instance_id: Option<String>,
    pub tmp_dir: String,
    pub modules: Vec<String>
}

impl Default for AlfredConfig {
    fn default() -> Self {
        Self {
            broker_url: DEFAULT_BROKER_URL.to_string(),
            username: None,
            password: None,
            ca_file: None,
            qos: DEFAULT_QOS,
            session_expiry_secs: DEFAULT_SESSION_EXPIRY_SECS,
            instance_id: None,
            tmp_dir: DEFAULT_TMP_DIR.to_string(),
            modules: Vec::new(),
        }
    }
}

impl fmt::Debug for AlfredConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlfredConfig")
            .field("broker_url", &self.broker_url)
            .field("username", &self.username)
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .field("ca_file", &self.ca_file)
            .field("qos", &self.qos)
            .field("session_expiry_secs", &self.session_expiry_secs)
            .field("instance_id", &self.instance_id)
            .field("tmp_dir", &self.tmp_dir)
            .field("modules", &self.modules)
            .finish()
    }
}

#[derive(Deserialize)]
struct FromFileConfig {
    alfred: FromFileAlfredConfig
}

impl FromFileConfig {

    fn get_config_filename() -> String {
        std::env::var("ALFRED_CONFIG")
            .ok()
            .and_then(|path| Path::new(&path.as_str()).exists().then_some(path))
            .unwrap_or_else(|| CONFIG_FILENAME.to_string())
    }

    fn read() -> Self {
        let contents = fs::read_to_string(Self::get_config_filename()).expect("Could not read file");
        toml::from_str(&contents).expect("Unable to load data")
    }
}

#[derive(Deserialize)]
struct FromFileAlfredConfig {
    broker_url: Option<String>,
    username: Option<String>,
    password: Option<String>,
    ca_file: Option<String>,
    qos: Option<u8>,
    session_expiry_secs: Option<u32>,
    instance_id: Option<String>,
    tmp_dir: Option<String>,
    #[serde(default)]
    modules: Vec<String>,
    // ZeroMQ keys, only used to detect an old configuration
    url: Option<Value>,
    pub_port: Option<Value>,
    sub_port: Option<Value>,
}

struct EnvConfig {
    alfred: EnvAlfredConfig,
}
impl EnvConfig {

    fn from_env() -> Self {
        Self {
            alfred: EnvAlfredConfig::init_from_env().expect("Error during initialization using env variable"),
        }
    }
}

#[derive(Envconfig)]
struct EnvAlfredConfig {
    #[envconfig(from = "ALFRED_BROKER_URL")]
    broker_url: Option<String>,
    #[envconfig(from = "ALFRED_USERNAME")]
    username: Option<String>,
    #[envconfig(from = "ALFRED_PASSWORD")]
    password: Option<String>,
    #[envconfig(from = "ALFRED_CA_FILE")]
    ca_file: Option<String>,
    #[envconfig(from = "ALFRED_QOS")]
    qos: Option<u8>,
    #[envconfig(from = "ALFRED_SESSION_EXPIRY_SECS")]
    session_expiry_secs: Option<u32>,
    #[envconfig(from = "ALFRED_INSTANCE_ID")]
    instance_id: Option<String>,
    #[envconfig(from = "ALFRED_TMP_DIR")]
    tmp_dir: Option<String>,
    // ZeroMQ variables, only used to detect an old configuration
    #[envconfig(from = "ALFRED_URL")]
    legacy_url: Option<String>,
    #[envconfig(from = "ALFRED_PUB_PORT")]
    legacy_pub_port: Option<String>,
    #[envconfig(from = "ALFRED_SUB_PORT")]
    legacy_sub_port: Option<String>,
}
