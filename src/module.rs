use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use clap::Command;
use crate::config::Config;
use crate::error::Error;
use crate::message::{Message, MessageType};
use crate::connection::{Connection, MODULE_INFO_TOPIC_REQUEST, MODULE_INFO_TOPIC_RESPONSE, TOPIC_PREFIX};

static STREAM_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct ModuleDetails {
    module_name: &'static str,
    version: &'static str,
    config: Option<Config>,
    capabilities: BTreeMap<String, String>
}
impl ModuleDetails {
    pub fn builder() -> ModuleDetailsBuilder {
        ModuleDetailsBuilder::default()
    }
}

#[derive(Default)]
#[must_use]
pub struct ModuleDetailsBuilder {
    module_name: &'static str,
    version: &'static str,
    config: Option<Config>,
    capabilities: BTreeMap<String, String>
}

impl ModuleDetailsBuilder {
    pub const fn new() -> Self {
        Self {
            module_name: "",
            version: "",
            config: None,
            capabilities: BTreeMap::new()
        }
    }
    pub const fn module_name(mut self, module_name: &'static str) -> Self {
        self.module_name = module_name;
        self
    }
    pub const fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }
    pub fn config(mut self, config: Option<Config>) -> Self {
        self.config = config;
        self
    }
    pub fn capabilities(mut self, capabilities: BTreeMap<String, String>) -> Self {
        self.capabilities = capabilities;
        self
    }
    pub fn build(self) -> ModuleDetails {
        ModuleDetails {
            module_name: self.module_name,
            version: self.module_name,
            config: self.config,
            capabilities: self.capabilities
        }
    }
}

pub struct AlfredModule {
    pub module_name: String,
    pub version: String,
    pub config: Config,
    pub connection: Connection,
    pub capabilities: BTreeMap<String, String> // TODO: change to HashMap<&'static str, &'static str>
}

impl AlfredModule {

    pub fn manage_args(app_name: &'static str, version: &'static str) {
        Command::new(app_name).version(version).get_matches();
    }
    
    pub const fn get_lib_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    pub async fn new(module_name: &'static str, version: &'static str) -> Result<Self, Error> {
        let module_config = ModuleDetails {
            module_name,
            version,
            config: None,
            capabilities: BTreeMap::new()
        };
        Self::new_with_details(module_config).await
    }

    pub async fn new_with_details(module_details: ModuleDetails) -> Result<Self, Error> {
        Self::manage_args(module_details.module_name, module_details.version);
        let config = module_details.config.unwrap_or_else(|| Config::read(Some(module_details.module_name)));
        let capabilities = module_details.capabilities;
        let mut connection = Connection::new(&config).await?;
        connection.listen(MODULE_INFO_TOPIC_REQUEST).await?;
        let alfred_module = Self {
            module_name: module_details.module_name.to_string(),
            version: module_details.version.to_string(),
            config,
            connection,
            capabilities
        };
        alfred_module.send(MODULE_INFO_TOPIC_RESPONSE, &alfred_module.get_info_message()).await?;
        Ok(alfred_module)
    }

    pub fn get_info_message(&self) -> Message {
        Message {
            text: self.module_name.clone(),
            message_type: MessageType::ModuleInfo,
            params: self.capabilities.clone(),
            ..Message::default()
        }
    }

    pub async fn listen(&mut self, topic: &str) -> Result<(), Error> {
        self.capabilities.insert(String::from(TOPIC_PREFIX), String::from(topic));
        self.connection.listen(topic).await
    }

    pub async fn receive(&self) -> Result<(String, Message), Error> {
        self.connection.receive(&self.module_name, &self.capabilities).await
    }

    pub async fn send(&self, topic: &str, message: &Message) -> Result<(), Error> {
        self.connection.send(topic, message).await
    }

    /// Sends one chunk of a text stream to `topic`. If `stream_id` is empty, a new one is
    /// generated for this stream; pass the same (non-empty) `stream_id` back in for every
    /// following chunk. Returns the `stream_id` used, so the caller can carry it forward.
    pub async fn send_stream(&self, topic: &str, text: String, sequence: u32, stream_id: String) -> Result<String, Error> {
        let stream_id = resolve_stream_id(&self.module_name, stream_id);
        let message = Message {
            text,
            message_type: MessageType::StreamText,
            stream_id: stream_id.clone(),
            sequence,
            ..Message::default()
        };
        self.send(topic, &message).await?;
        Ok(stream_id)
    }

    pub async fn send_event(&mut self, publisher_name: &str, event_name: &str, message: &Message) -> Result<(), Error> {
        self.connection.send_event(publisher_name, event_name, message).await
    }
}

fn resolve_stream_id(module_name: &str, stream_id: String) -> String {
    if stream_id.is_empty() { generate_stream_id(module_name) } else { stream_id }
}

fn generate_stream_id(module_name: &str) -> String {
    let counter = STREAM_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    format!("{module_name}-{nanos}-{counter}")
}

#[cfg(test)]
mod tests {
    use super::resolve_stream_id;

    #[test]
    fn resolve_stream_id_keeps_supplied_id() {
        assert_eq!(resolve_stream_id("mod", String::from("existing-id")), "existing-id");
    }

    #[test]
    fn resolve_stream_id_generates_when_empty() {
        let generated = resolve_stream_id("mod", String::new());
        assert!(!generated.is_empty());
        assert!(generated.starts_with("mod-"));
    }

    #[test]
    fn resolve_stream_id_generates_distinct_ids() {
        let first = resolve_stream_id("mod", String::new());
        let second = resolve_stream_id("mod", String::new());
        assert_ne!(first, second);
    }
}