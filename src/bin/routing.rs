use std::collections::HashMap;
use std::error::Error;
use std::fs;
use serde_derive::Deserialize;
use alfred_rs::log::{debug, info, warn};
use alfred_rs::AlfredModule;
use alfred_rs::config_message::ConfigMessage;

const ROUTING_FILENAME: &str = "routing.toml";

#[derive(Deserialize)]
#[derive(Debug)]
pub struct RoutingItem {
    pub from_topic: String,
    pub to_topic: String,
    pub message: Option<ConfigMessage>
}

#[derive(Deserialize)]
#[derive(Debug)]
pub struct Routing {
    pub routing: Vec<RoutingItem>
}

impl Routing {
    pub fn from_file() -> Result<Self, Box<dyn Error>>{
        let contents = fs::read_to_string(ROUTING_FILENAME)?;
        toml::from_str(&contents).map_err(Into::into)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Loading routing module...");
    let routing_config = Routing::from_file().ok();
    if routing_config.is_none() {
        warn!("No routing found. Exiting.");
        return Ok(());
    }
    let routing_config = routing_config.expect("No routing config found");
    if routing_config.routing.is_empty() {
        warn!("Routing config is empty. Exiting.");
        return Ok(());
    }
    let mut module = AlfredModule::new("routing", env!("CARGO_PKG_VERSION")).await?;

    let mut routing_hashmap = HashMap::new();
    for routing in routing_config.routing {
        debug!("{} -> {}", routing.from_topic, routing.to_topic.clone());
        module.listen(routing.from_topic.as_str()).await?;
        if !routing_hashmap.contains_key(&routing.from_topic) {
            routing_hashmap.insert(routing.from_topic.clone(), vec![]);
        }
        routing_hashmap
            .get_mut(&routing.from_topic)
            .unwrap_or_else(|| panic!("Unable to insert new routing with key {}", routing.from_topic))
            .push(routing);
    }
    loop {
        let (topic, message) = module.receive().await?;
        let routing_items = routing_hashmap.get(&topic);

        for routing_item in routing_items.unwrap_or(&Vec::new()) {
            let routing_message = routing_item.message.clone()
                .map_or_else(|| message.clone(), |routing_message| routing_message.generate_message(&message));
            module.send(routing_item.to_topic.as_str(), &routing_message).await?;
        }
    }
}
