use std::collections::HashMap;
use std::error::Error;
use alfred_rs::log::{debug, info, warn};
use alfred_rs::connection::{Receiver, Sender};
use alfred_rs::interface_module::InterfaceModule;
use alfred_rs::router_config::Routing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Loading routing module...");
    let routing_config = Routing::from_file().map_or(None, |conf| Some(conf));
    if routing_config.is_none() {
        warn!("No routing found. Exiting.");
        return Ok(());
    }
    let routing_config = routing_config.unwrap();
    if routing_config.routing.len() == 0 {
        warn!("Routing config is empty. Exiting.");
        return Ok(());
    }
    let mut module = InterfaceModule::new(String::from("routing")).await?;

    let mut routing_hashmap = HashMap::new();
    for routing in routing_config.routing {
        debug!("{} -> {}", routing.from_topic.clone(), routing.to_topic.clone());
        module.listen(routing.from_topic.clone()).await?;
        routing_hashmap.insert(routing.from_topic.clone(), routing);
    }
    loop {
        let (topic, message) = module.receive().await?;
        let routing_item = routing_hashmap.get(&topic);
        if routing_item.is_none() {
            continue;
        }
        let routing_item = routing_item.unwrap();
        let routing_message = routing_item.message.clone()
            .map(|routing_message| routing_message.generate_message(&message))
            .unwrap_or(message);
        module.send(routing_item.to_topic.clone(), &routing_message).await?
    }
}