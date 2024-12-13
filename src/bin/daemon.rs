use log::info;
use zmq2::Context;
use alfred_rs::AlfredModule;

fn main() {
    env_logger::init();
    AlfredModule::manage_args("daemon", env!("CARGO_PKG_VERSION"));
    info!("Loading daemon...");
    let config = alfred_rs::config::Config::read(None);
    let pub_port = config.alfred.pub_port;
    let sub_port = config.alfred.sub_port;
    let context = Context::new();
    let xpub_sub = context.socket(zmq2::XPUB).expect("Failed to create XPUB socket");
    let pub_xsub = context.socket(zmq2::XSUB).expect("Failed to create XSUB socket");
    info!("Binding port {} for publish...", pub_port);
    pub_xsub.bind(format!("tcp://*:{pub_port}").as_str()).expect("Failed to bind XPUB socket");
    info!("Binding port {} for subscription...", sub_port);
    xpub_sub.bind(format!("tcp://*:{sub_port}").as_str()).expect("Failed to bind XSUB socket");

    zmq2::proxy(&pub_xsub, &xpub_sub).expect("Failed starting zmq proxy");
}
