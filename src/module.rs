use std::collections::HashMap;
use clap::Command;
use crate::config::Config;
use crate::error::Error;
use crate::message::{Message, MessageType};
use crate::connection::{Connection, MODULE_INFO_TOPIC_REQUEST, MODULE_INFO_TOPIC_RESPONSE, TOPIC_PREFIX};

pub struct ModuleDetails {
    module_name: &'static str,
    version: &'static str,
    config: Option<Config>,
    capabilities: HashMap<String, String>
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
    capabilities: HashMap<String, String>
}

impl ModuleDetailsBuilder {
    pub fn new() -> Self {
        Self {
            module_name: "",
            version: "",
            config: None,
            capabilities: HashMap::new()
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
    pub fn capabilities(mut self, capabilities: HashMap<String, String>) -> Self {
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
    pub capabilities: HashMap<String, String> // TODO: change to HashMap<&'static str, &'static str>
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
            capabilities: HashMap::new()
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

    pub async fn send_event(&mut self, publisher_name: &str, event_name: &str, message: &Message) -> Result<(), Error> {
        self.connection.send_event(publisher_name, event_name, message).await
    }
}