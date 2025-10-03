// 導入 Zenoh 通訊及非同步操作所需的模組
use zenoh::Config;
use tokio::time::{sleep, Duration};

/// Zenoh 多服務節點的主要進入點
/// 此應用展示各種 Zenoh 模式：
/// - Queryable 服務（echo 和 convert）
/// - 發佈/訂閱模式（Pub/Sub）
/// - 客戶端查詢
#[tokio::main]
async fn main() {
    // 使用預設設定初始化 Zenoh session
    // Session 是所有 Zenoh 操作的主要入口
    let session = zenoh::open(Config::default()).await.unwrap();
    println!("Zenoh 多服務節點啟動！");

    // -----------------------------
    // Echo 服務
    // -----------------------------
    // 宣告一個可查詢服務，回應 "service/echo" 的請求
    // 此服務會簡單地回傳收到的訊息
    let echo = session.declare_queryable("service/echo").await.unwrap();
    tokio::spawn(async move {
        // 持續監聽傳入的查詢
        while let Ok(query) = echo.recv_async().await {
            // 從查詢中提取訊息內容
            let msg = query.payload().map(|p| p.try_to_string().unwrap_or_default().to_string()).unwrap_or_default();
            println!("[Echo 服務] 收到: {}", msg);

            // 回應查詢，回傳 echo 後的訊息
            query.reply(query.key_expr().clone(), format!("Echo: {}", msg))
                .await.unwrap();
        }
    });

    // -----------------------------
    // 二進位轉換服務
    // -----------------------------
    // 宣告一個可查詢服務，將數字轉換為二進位格式
    let convert = session.declare_queryable("service/convert").await.unwrap();
    tokio::spawn(async move {
        // 持續監聽傳入的查詢
        while let Ok(query) = convert.recv_async().await {
            // 從查詢中提取訊息內容
            let msg = query.payload().map(|p| p.try_to_string().unwrap_or_default().to_string()).unwrap_or_default();
            println!("[Binary Convert 服務] 收到: {}", msg);

            // 嘗試解析訊息為整數，並轉換為二進位
            // 若解析失敗，回傳錯誤訊息
            let reply = msg.parse::<i64>()
                .map(|v| format!("{} 的二進位格式為 0b{:b}", v, v))
                .unwrap_or_else(|_| "錯誤：不是有效的整數".into());

            // 回應查詢，傳回二進位轉換結果
            query.reply(query.key_expr().clone(), reply).await.unwrap();
        }
    });

    // -----------------------------
    // Sensor 發佈者
    // -----------------------------
    // 宣告一個發佈者，用於模擬溫度感測器資料
    // 此發佈者將溫度資料發佈到 "sensor/temperature" 主題
    let publisher = session.declare_publisher("sensor/temperature").await.unwrap();
    tokio::spawn(async move {
        // 初始化溫度值
        let mut value = 25.0;
        loop {
            // 格式化並發佈溫度資料
            let msg = format!("Temp = {:.1}", value);
            publisher.put(msg.clone()).await.unwrap();
            println!("[Publisher] 發佈: {}", msg);

            // 模擬溫度變化，略微增加
            value += 0.1;
            // 等待 2 秒再發佈下一筆資料
            sleep(Duration::from_secs(2)).await;
        }
    });

    // -----------------------------
    // 訂閱者
    // -----------------------------
    // 宣告一個訂閱者，監聽所有感測器資料，使用通配符模式
    // "sensor/**" 會匹配任何以 "sensor/" 開頭的 key expression
    let subscriber = session.declare_subscriber("sensor/**").await.unwrap();
    tokio::spawn(async move {
        // 持續監聽感測器主題的發佈資料
        while let Ok(sample) = subscriber.recv_async().await {
            // 取出 payload 並顯示對應的主題 key
            let payload = sample.payload().try_to_string().unwrap_or_default().to_string();
            println!("[Subscriber] '{}' -> {}", sample.key_expr(), payload);
        }
    });

    // -----------------------------
    // 客戶端定期查詢
    // -----------------------------
    // 建立一個客戶端任務，定期查詢服務
    // 展示 Zenoh 的請求-回應模式
    tokio::spawn({
        let session = session.clone();
        async move {
            // 等待服務啟動完成，再發送查詢
            sleep(Duration::from_secs(5)).await;

            let mut counter = 0;
            loop {
                counter += 1;
                println!("[Client] 發送查詢 #{}", counter);

                // 查詢 echo 服務
                let replies = session.get("service/echo")
                    .payload(format!("Hello Zenoh! #{}", counter))
                    .await.unwrap();

                // 處理所有 echo 服務回覆
                while let Ok(reply) = replies.recv_async().await {
                    if let Ok(sample) = reply.result() {
                        println!("[Client] Echo 回覆: {}", sample.payload().try_to_string().unwrap());
                    }
                }

                // 遞增整數並查詢 convert 服務
                let test_value = 42 + counter;
                let replies = session.get("service/convert")
                    .payload(test_value.to_string())
                    .await.unwrap();

                // 處理 convert 服務所有回覆
                while let Ok(reply) = replies.recv_async().await {
                    if let Ok(sample) = reply.result() {
                        println!("[Client] Convert 回覆: {}", sample.payload().try_to_string().unwrap());
                    }
                }

                // 等待 3 秒後再發送下一批查詢
                sleep(Duration::from_secs(3)).await;
            }
        }
    });

    // 保持主執行緒存活，以允許所有 spawned 任務持續運行
    // 無限迴圈防止程式結束
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}
