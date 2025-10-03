use zenoh::Config;
#[tokio::main]
async fn main() {
    let session = zenoh::open(Config::default()).await.unwrap();
    println!("Convert service started!");

    let convert = session.declare_queryable("service/convert").await.unwrap();
    tokio::spawn(async move {
        while let Ok(query) = convert.recv_async().await {
            let msg = query.payload()
                .map(|p| p.try_to_string().unwrap_or_default())
                .unwrap_or_default();
            println!("[Convert service] Received request: {}", msg);
            let reply = msg.parse::<i64>()
                .map(|v| format!("0b{:b}", v))
                .unwrap_or_else(|_| "Error: not an integer".into());
            query.reply(query.key_expr().clone(), reply).await.unwrap();
        }
    });

    loop { tokio::time::sleep(std::time::Duration::from_secs(60)).await; }
}
