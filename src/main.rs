use alfred_mq::{Connection, Publisher, Subscriber};
use alfred_mq::message::MessageType;
use rand::Rng;

fn test() -> String {
    let mut rng = rand::thread_rng();
    return format!("value: {}", rng.gen::<f64>());
}
async fn subscriber_example() {
    println!("Connecting...");
    let mut connection = Connection::new().await
        .expect("Error on connect as subscriber");
    println!("Connected");
    println!("Subscribing...");
    connection.subscribe("topic".to_string()).await.expect("Error on subscribe");
    println!("Subscribed");
    println!("Waiting for msgs...");
    // let string = response.compress();
    // let compress = string.clone();
    // let x = compress.as_str();
    // let message_compressed = x.clone();
    // while let Some(message) = connection.get_message().await.into() {
    //     let vec1 = vec!["TOPIC", message_compressed];
    //     // connection.publisher.send(vec!["TOPIC", message_compressed].into()).await.expect("Error on publish");
    //     connection.publisher.send_multipart(vec1, 0).expect("Error on send");
    //
    //     /*publish2(&mut connection.publisher, "TOPIC", message_compressed).await
    //         .expect("Error on publish");*/
    //     //connection.publish_str("topic", &compressed.as_str()).await;
    //     //connection.publish(&PubMessage::new("topic", compressed.as_str())).await;
    //     println!("{:?}", message)
    // }
    // let string = response.compress();
    // let compress = string.clone();
    // let x = compress.as_str();
    // let message_compressed = x.clone();
    while let Some(message) = connection.get_message().await.into() {
        let response: alfred_mq::message::Message = alfred_mq::message::Message {
            text: "text".to_string(),
            starting_module: "starting".to_string(),
            request_topic: "request_topic".to_string(),
            response_topic: "response_topic".to_string(),
            sender: "me".to_string(),
            message_type: MessageType::TEXT,
            params: Default::default(),
        };
        println!("{:?}", message);
        connection.publish("echo".to_string(), &response).await.expect("Error on publish");
        println!("{:?}", response);
    }


}

#[tokio::main]
async fn main() {
    subscriber_example().await;
}