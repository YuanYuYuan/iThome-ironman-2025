use zenoh::Config;
#[tokio::main]
async fn main() {
    let session = zenoh::open(Config::default()).await.unwrap();
    let subscriber = session.declare_subscriber("sensor/**").await.unwrap();

    while let Ok(sample) = subscriber.recv_async().await {
        let payload = sample.payload().try_to_string().unwrap_or_default().to_string();
        println!("[Subscriber] '{}' -> {}", sample.key_expr(), payload);
    }
}
