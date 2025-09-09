# 介紹Zenoh 的同步 API、Get/Queryable 與Keyexpr


延續昨天的內容，我們將應用程式擴充至使用 **同步 API**，並展示透過 **callback**（回呼函數）來提升效能的方式。以下是筆者節錄自官方範例程式碼 [Zenoh examples](https://github.com/eclipse-zenoh/zenoh/blob/main/examples)。

---

## 同步 API 與 Callback 介紹

在 Zenoh 中，Publisher（發佈者）可以使用 **同步的 `put`** 呼叫來送出資料，而 Subscriber（訂閱者）則能透過 **callback** 在訊息到達時即時處理，避免不必要的等待與輪詢。

### Publisher 範例 (`z_pub_thr.rs`)

```rust
let session = zenoh::open(args.common).wait().unwrap();

let publisher = session
    .declare_publisher("test/thr")
    // QoS: 當傳輸佇列滿時的行為
    .congestion_control(CongestionControl::Block)
    // QoS: 訊息的優先級
    .priority(Priority::DEFAULT)
    .wait()
    .unwrap();

// 發佈迴圈
loop {
    publisher.put(data.clone()).wait().unwrap(); // 同步送出
}
```

#### QoS 說明

* **Congestion Control（壅塞控制）**
  當傳輸佇列已滿時，Zenoh 的行為設定：

  * `Drop` → 直接丟棄新訊息。
  * `Block` → 等待直到佇列可用空間（適合可靠傳輸）。
  * `BlockFirst` → 只針對第一筆訊息等待，之後的直接丟棄。常用於 **狀態更新**（只需要最新值）。

* **Priority（優先級）**
  Zenoh 為每個優先級維護一個佇列(Zenoh一共提供八個可供客製化)，並按照優先級順序服務：

  * `RealTime` → 最緊急（控制迴路）。
  * `InteractiveHigh` / `InteractiveLow` → 人機互動、操作指令。
  * `DataHigh`, `Data`（預設值）, `DataLow` → 一般資料傳輸。
  * `Background` → 最低優先級（如日誌、批次上傳）。

### QoS 使用情境

| 設定                              | 常見應用          | 原因               |
| ------------------------------- | ------------- | ---------------- |
| `CongestionControl::Drop`       | 視訊串流、快速感測器資料  | 可容忍掉幀，避免延遲累積。    |
| `CongestionControl::Block`      | 關鍵控制訊號     | 確保資料不遺失，即使延遲增加。  |
| `CongestionControl::BlockFirst` | 機器人姿態、圖資更新 | 只需要最新狀態，舊資料可忽略。  |
| `Priority::RealTime`            | 安全控制訊號        | 必須高於一切。          |
| `Priority::InteractiveHigh`     | 操作者指令、UI 操作   | 低延遲回應，確保即時互動。    |
| `Priority::Data`                | 一般遙測、系統監控     | 大部分應用的平衡點。       |
| `Priority::Background`          | 批次資料、日誌上傳     | 只利用剩餘頻寬，不影響其他流量。 |

---

### Subscriber 範例 (`z_sub_thr.rs`)

```rust
let session = zenoh::open(config).wait().unwrap();

let mut stats = Stats::new(n);
session
    .declare_subscriber("test/thr")
    .callback_mut(move |_sample| {
        stats.increment();
        if stats.finished_rounds >= m {
            std::process::exit(0)
        }
    })
    .background()
    .wait()
    .unwrap();

std::thread::park(); // 保持程式運行
```

使用 `callback_mut`，每當有訊息進入就會即時觸發邏輯，而非主動去輪詢，大幅提升效能。

---

## z\_get 與 z\_queryable

### `z_get.rs`

* **角色**：用戶端主動向網路發出 **查詢**，等待 Queryable 回覆。
* **流程**：建立查詢 → 發送 → 收到多個回覆。

```rust
let replies = session
    .get(&selector)
    .target(target)
    .timeout(timeout)
    .await
    .unwrap();

while let Ok(reply) = replies.recv_async().await {
    if let Ok(sample) = reply.result() {
        println!(">> Received {} = {}", sample.key_expr(), sample.payload().try_to_string().unwrap());
    }
}
```

---

### `z_queryable.rs`

* **角色**：伺服端（或資料持有者），等待查詢請求並回覆。
* **流程**：宣告 Queryable → 等待查詢 → 處理 payload → 回覆結果。

```rust
let queryable = session
    .declare_queryable(&key_expr)
    .complete(complete)
    .await
    .unwrap();

while let Ok(query) = queryable.recv_async().await {
    query.reply(key_expr.clone(), payload.clone()).await.unwrap();
}
```

---

### Pub/Sub 與 Get/Queryable 的差異

* **Pub/Sub** → 資料推送模式（資料持續流動，例如感測器數據）。
* **Get/Queryable** → 查詢模式（用戶端請求一次，伺服端回應一次，類似 RPC）。

| 模式            | 特點          | 適用場景              |
| ------------- | ----------- | ----------------- |
| Pub/Sub       | 發布即送出，訂閱即收到 | 感測器數據、即時串流        |
| Get/Queryable | 主動查詢才有回覆    | 參數查詢、資料庫存取、RPC 呼叫 |

---

## Zenoh 抽象說明

### Key Expression（鍵表達式）

與 MQTT/DDS 使用的「Topic」不同，Zenoh 使用 **Key Expression**，可用萬用字元進行模式匹配：

* `*` → 匹配單一區段
* `$*` → 區段內的子字串
* `**` → 匹配任意層級

範例：

* `a/*/b` → 匹配 `a/c/b`，但不匹配 `a/c/d/b`。
* `a/**/b` → 匹配 `a/c/b`、`a/c/d/b`、`a/b`。
* `a/$*c$*/b` → 匹配 `a/xcy/b`、`a/cool/b`、`a/c/b`。

---

### Selector（選擇器）

Selector 是 **Key Expression + 參數** 的組合，用來在查詢時攜帶額外參數（類似 URL 查詢字串）。

範例：

```
path/**/something?arg1=val1;arg2=value%202
```

* `path/**/something` → Key Expression
* `?arg1=val1;arg2=value%202` → 查詢參數

用途：

* 傳遞 RPC 的參數
* 精確指定查詢條件
* 與 REST API 相容（Zenoh Router 可直接透過 HTTP 呼叫 Selector）

---

## 結論

今天介紹了 Zenoh **同步 API** 的使用方式，並展示如何透過 **callback** 提升訂閱效能。同時，解釋了 **QoS 機制**（壅塞控制與優先級），並提供實際應用情境。最後，我們也介紹了 **Get/Queryable 查詢模式** 與 **Pub/Sub 推送模式** 的差異，以及 Zenoh 的抽象設計：**Key Expression** 與 **Selector**。

Zenoh 不僅支援高效能資料傳輸，也提供靈活的資料查詢與命名抽象，能同時滿足 **即時性需求** 與 **資料檢索需求**，在物聯網、機器人、分散式系統中皆十分實用。
