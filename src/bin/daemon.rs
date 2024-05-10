use zmq2::Context;

fn main() {
    let context = Context::new();
    let xpub_sub = context.socket(zmq2::XPUB).unwrap();
    let pub_xsub = context.socket(zmq2::XSUB).unwrap();
    pub_xsub.bind(format!("tcp://*:{}", alfred_rs::connection::PUB_PORT).as_str()).unwrap();
    xpub_sub.bind(format!("tcp://*:{}", alfred_rs::connection::SUB_PORT).as_str()).unwrap();

    zmq2::proxy(&pub_xsub, &xpub_sub).unwrap();
}
