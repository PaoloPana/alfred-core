use log::{debug, info};
use alfred_core::AlfredModule;
use alfred_core::error::Error;
use alfred_core::message::MessageType;

const MODULE_NAME: &str = "logs";
const WILDCARD_TOPIC: &str = "";
const MAX_CHARS_TEXT: usize = 20;

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
            MessageType::StreamText | MessageType::StreamAudio | MessageType::StreamPhoto => {
                let message_text = truncate(message.text.as_str(), MAX_CHARS_TEXT);
                if message.is_final {
                    info!("{}[{}][stream_id: {}][sequence: {}][final]: {}", topic, message.message_type, message.stream_id, message.sequence, message_text);
                } else {
                    info!("{}[{}][stream_id: {}][sequence: {}]: {}", topic, message.message_type, message.stream_id, message.sequence, message_text);
                }
            },
            MessageType::ModuleInfo => {
                info!("Module Info: {}\n\t{:?}", message.text, message.params);
            }
        }
        debug!("response_topics: {:?}", message.response_topics);
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => s.to_string(),
        Some((idx, _)) => s[..idx].to_string() + "...",
    }
}