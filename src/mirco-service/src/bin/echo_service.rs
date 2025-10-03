use zenoh::Config;
#[tokio::main]
async fn main() {
    let session = zenoh::open(Config::default()).await.unwrap();
    println!("Echo service started!");

    let echo = session.declare_queryable("service/echo").await.unwrap();
    tokio::spawn(async move {
        while let Ok(query) = echo.recv_async().await {
            let msg = query.payload()
                .map(|p| p.try_to_string().unwrap_or_default())
                .unwrap_or_default();
            println!("[Echo service] Received request: {}", msg);
            query.reply(query.key_expr().clone(), format!("Echo: {}", msg)).await.unwrap();
        }
    });

    loop { tokio::time::sleep(std::time::Duration::from_secs(60)).await; }
}