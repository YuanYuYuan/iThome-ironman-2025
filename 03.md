# 開始使用 Zenoh 與 Rust：從安裝到高效能 Pub/Sub

> Zenoh 的核心哲學是「資料為中心」，無論是在 **IoT 裝置、邊緣運算節點**，還是 **雲端環境**，都能用相同的協議無縫存取資料。
> 本文將帶你從 **安裝 Zenoh** 到撰寫 **第一個 Hello Zenoh 專案**

---

## 1. 為什麼選擇 Zenoh？

在分散式系統中，傳統的 Request/Response 模式耦合度高、效能有限。
**Pub/Sub** 解耦了資料的發佈與接收，使得系統具備更佳的即時性與擴展性。

Zenoh 更進一步：

* 提供 **低延遲、高吞吐量**
* 支援 **跨網路、跨平台的拓撲**
* 同時具備 **Pub/Sub + Query + Storage** 功能

非常適合 **IoT、邊緣運算、雲端混合場景**。

---

## 2. 安裝 Zenoh 與建立 Rust 專案

### 安裝 Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

### 建立專案

```bash
cargo new hello-zenoh
cd hello-zenoh
```

### 加入依賴

在 `Cargo.toml` 中加入：

```toml
[dependencies]
tokio = { version = "1.47.1" }
zenoh = "1.5.1"
```

---

## 3. Zenoh 支援的平台

Zenoh 的一大優勢是「跨平台、跨語言」支援：

* **作業系統**：Linux、Windows、macOS、FreeRTOS、Zephyr
* **硬體架構**：x86、ARM、RISC-V
* **語言綁定**：Rust、C、C++、Python、Go、Java
* **部署模式**：雲端、邊緣、嵌入式微控制器

這意味著你在 **樹莓派**、**工業電腦**、甚至 **微控制器** 上都能運行相同的 Zenoh 應用，並且與雲端無縫銜接。
筆者將在後續詳細的討論Zenoh是怎麼做到這些支援的，敬請期待！

---

## 4. Hello Zenoh：Rust Pub/Sub 範例

### Subscriber

_bin/z_sub.rs_
```rust
use tokio; // 匯入 tokio 非同步執行環境

#[tokio::main] // 使用 tokio 執行非同步 main 函式
async fn main() {
    // 啟動 Zenoh 日誌系統 (可透過環境變數控制輸出層級，預設為 "error")
    zenoh::init_log_from_env_or("error");

    // 建立與 Zenoh 的 Session，用於連線 Zenoh 網路
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();

    // 宣告要訂閱的 Key Expression（類似於 MQTT 的 topic）
    let key_expr = "demo/example/hello";
    let subscriber = session.declare_subscriber(key_expr).await.unwrap();

    println!("Subscribed on '{key_expr}'...");

    // 進入迴圈，持續非同步接收 Publisher 發佈的訊息
    while let Ok(sample) = subscriber.recv_async().await {
        // 嘗試將 Payload 轉換成字串，若失敗則回傳空字串
        let payload = sample.payload().try_to_string().unwrap_or_default();
        // 輸出接收到的訊息內容與其對應的 Key Expression
        println!("Received: {} = {}", sample.key_expr(), payload);
    }
}
```

### Publisher

_bin/z_pub.rs_
```rust
use std::time::Duration; // 匯入 Duration 用於設定延遲時間
use tokio;                // 匯入 tokio 非同步執行環境

#[tokio::main] // 使用 tokio 的非同步 main 函式
async fn main() {
    // 初始化 Zenoh 日誌系統，若環境變數未設定，預設為 "error" 級別
    zenoh::init_log_from_env_or("error");

    // 建立 Zenoh Session，連線到預設配置的 Zenoh 網路
    let session = zenoh::open(zenoh::Config::default()).await.unwrap();

    // 設定要發佈的 Key Expression（類似 topic）
    let key_expr = "demo/example/hello";

    // 宣告 Publisher，用於將訊息發佈到指定 Key Expression
    let publisher = session.declare_publisher(key_expr).await.unwrap();

    println!("Publishing on '{key_expr}'...");

    // 無限迴圈，每秒發佈一次訊息
    for idx in 0.. {
        tokio::time::sleep(Duration::from_secs(1)).await; // 暫停 1 秒
        let buf = format!("Hello Zenoh! #{idx}"); // 建立訊息內容
        publisher.put(buf).await.unwrap(); // 發佈訊息
    }
}
```

---


## 8. 總結

今天我們完成了：

1. 使用 Zenoh v**1.5.1** 建立 Rust 專案
2. 撰寫並解釋第一個 Hello Zenoh Pub/Sub 範例
