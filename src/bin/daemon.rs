use std::error::Error;
use log::info;
#[cfg(feature = "daemon")]
use zmq2::Context;

#[cfg(feature = "daemon")]
fn main() -> Result<(), Box<dyn Error>>{
    env_logger::init();
    info!("Loading daemon...");
    let config = alfred_rs::config::Config::read(None)?;
    let pub_port = config.alfred.pub_port;
    let sub_port = config.alfred.sub_port;
    let context = Context::new();
    let xpub_sub = context.socket(zmq2::XPUB).unwrap();
    let pub_xsub = context.socket(zmq2::XSUB).unwrap();
    info!("Binding port {} for publish...", pub_port);
    pub_xsub.bind(format!("tcp://*:{}", pub_port).as_str()).unwrap();
    info!("Binding port {} for subscription...", sub_port);
    xpub_sub.bind(format!("tcp://*:{}", sub_port).as_str()).unwrap();

    zmq2::proxy(&pub_xsub, &xpub_sub).unwrap();
    Ok(())
}
