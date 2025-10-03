use zenoh::Config;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let session = zenoh::open(Config::default()).await.unwrap();
    let publisher = session.declare_publisher("sensor/temperature").await.unwrap();
    let mut value = 25.0;

    loop {
        let msg = format!("Temp = {:.1}", value);
        publisher.put(msg.clone()).await.unwrap();
        println!("[Publisher] {}", msg);
        value += 0.1;
        sleep(Duration::from_secs(2)).await;
    }
}
