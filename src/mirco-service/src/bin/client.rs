use zenoh::Config;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // let session = zenoh::open(Config::default()).await.unwrap();

    let config = Config::from_file("config.json5").unwrap();
    let session = zenoh::open(config).await.unwrap();

    sleep(Duration::from_secs(5)).await; // wait for services

    let mut counter = 0;
    loop {
        counter += 1;
        let replies = session.get("service/echo")
            .payload(format!("Hello Zenoh! #{}", counter))
            .await.unwrap();
        while let Ok(reply) = replies.recv_async().await {
            if let Ok(sample) = reply.result() {
                println!("[Client] Echo reply: {}", sample.payload().try_to_string().unwrap());
            }
        }

        let test_value = 42 + counter;
        let replies = session.get("service/convert")
            .payload(test_value.to_string())
            .await.unwrap();
        while let Ok(reply) = replies.recv_async().await {
            if let Ok(sample) = reply.result() {
                println!("[Client] Convert reply: {}", sample.payload().try_to_string().unwrap());
            }
        }

        sleep(Duration::from_secs(3)).await;
    }
}
