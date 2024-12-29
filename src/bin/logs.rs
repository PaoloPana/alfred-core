use log::{debug, info};
use alfred_core::AlfredModule;
use alfred_core::error::Error;
use alfred_core::message::MessageType;

const MODULE_NAME: &str = "logs";
const WILDCARD_TOPIC: &str = "";

#[tokio::main]
#[allow(clippy::print_stdout,clippy::use_debug)]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let mut module = AlfredModule::new(MODULE_NAME, env!("CARGO_PKG_VERSION")).await?;
    module.listen(WILDCARD_TOPIC).await?;
    loop {
        let (topic, message) = module.connection.receive_all().await?;
        match message.message_type {
            MessageType::Text => {
                info!("{}: {}", topic, message.text);
            },
            MessageType::Unknown | MessageType::Audio | MessageType::Photo => {
                info!("{}[{}]: {}", topic, message.message_type, message.text);
            },
            MessageType::ModuleInfo => {
                info!("Module Info: {}\n\t{:?}", message.text, message.params);
            }
        }
        debug!("response_topics: {:?}", message.response_topics);
    }
}
