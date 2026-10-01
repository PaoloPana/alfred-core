use log::{debug, info, warn};
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
        let (topic, message) = match module.connection.receive_all().await {
            Err(Error::MessageEncodingError(err)) => {
                warn!("Message that cannot be decoded: {err}");
                continue;
            },
            received => received?
        };
        match message.message_type {
            MessageType::Text => {
                info!("{}: {}", topic, message.payload_description());
            },
            MessageType::StreamText | MessageType::StreamAudio | MessageType::StreamPhoto => {
                info!("{}[{}][stream_id: {}][sequence: {}][is_final: {}]: {}", topic, message.message_type, message.stream_id, message.sequence, message.is_final, message.payload_description());
            },
            MessageType::ModuleInfo => {
                info!("Module Info: {}\n\t{:?}", message.payload_description(), message.params);
            },
            MessageType::Unknown | MessageType::Audio | MessageType::Photo | _ => {
                info!("{}[{}]: {}", topic, message.message_type, message.payload_description());
            }
        }
        debug!("response_topics: {:?}", message.response_topics);
    }
}
