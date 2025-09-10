# Zenoh Router：打造跨協定、低開銷的資料骨幹

在上一篇文章中，我們介紹了 Zenoh 的資料交換基礎。今天，我們將焦點轉向 **網路層面** —— 特別是 **節點發現 (discovery)** 的運作方式，以及 Zenoh Router 如何開啟全新的可能性。

> Zenoh 可運行於任何網路拓撲，無論是 Broker 模式、Routed 模式，或是 Peer-to-Peer 模式，都能靈活支援。

![any topology](https://raw.githubusercontent.com/YuanYuYuan/iThome-ironman-2025/refs/heads/main/pic/any-topology.png)

---

## Discovery：超越 DDS 與 MQTT

在 **對等模式 (peer mode)** 下使用 Zenoh，並啟用 multicast discovery（預設啟用），節點能夠自動互相發現。這與 **MQTT** 完全不同，因為 MQTT 的所有客戶端都必須透過一個 broker 連線。

點對點 (P2P) 網路在 **區域網路 (LAN)** 中表現不錯，但跨越網際網路時就會遇到限制。而這正是 **DDS** 的痛點之一。

更糟的是，DDS 因其協定設計，會出現所謂的 **discovery storm**。根據 [這篇文章](https://zenoh.io/blog/2021-03-23-discovery/)，DDS 的 discovery 流量會隨著節點、讀取端與寫入端的數量，呈現 **平方級數增長**。

---

## DDS 的 Discovery Storm 問題

ROS2 採用 **DDS** 作為底層通訊。DDS 的 discovery 機制主要依賴兩個協定：

* **SPDP (Simple Participant Discovery Protocol)** → 發現所有 domain participants
* **SEDP (Simple Entity Discovery Protocol)** → 交換所有的 readers、writers 以及可選的 topics

最終結果是：在一個系統中，若有 `n` 個 participants，每個有 `r` 個 readers 和 `w` 個 writers，則 discovery 流量會隨以下公式增長：

```
n * (n-1) * (r + w)
```

這種平方級增長不只是理論問題，而是真實的網路負擔：

* **每個 participant 都必須存儲全部 discovery 資訊**，即使其中大部分對它並不相關。
* 隨著節點增加，流量持續膨脹。
* 在 **WiFi** 等無線網路上，頻寬極易被耗盡。

ROS2 開發者嘗試過如 **ROS2 Discovery Service** 等方式來緩解，但這些方案只能減輕壓力，無法解決根本問題。

---

## Zenoh 的不同設計

Zenoh 在設計時就考慮到了這些限制。它不會廣播所有 publishers/subscribers 的細節，而是僅交換 **資源需求 (resource interests)**。

主要改進包括：

1. **僅交換資源需求** —— 無需傳送所有 pub/sub 的細節。
2. **資源泛化 (generalization)** —— 可用萬用字元歸納多個資源，例如 `/mybot/**`。
3. **協定極度精簡** —— 訊息結構高度壓縮，降低線上流量。
4. **執行階段可靠性** —— 不依賴 reader-writer 細節，大幅降低開銷。

因此，Zenoh 的 discovery 流量在系統擴張時依然 **可預測且穩定**。

---

## 實驗研究：ROS2 Turtlebot SLAM

為驗證這些概念，進行了實地測試：

* **機器人**：Turtlebot Burger，執行 ROS2 Foxy
* **應用**：SLAM 與 RVIZ2
* **網路**：WiFi（NetGear NightHawk 路由器）

比較三種情境：

1. **僅使用 DDS**（基準）
2. **Zenoh 搭配 plugin-dds**，橋接 DDS 流量
3. **Zenoh 搭配資源泛化 (RG)** 與 **Warm Start (WS)**

---

### 結果

| 測試場景            | 封包數 | 平均大小 (B) | 總傳輸量 (Bytes) | 相對 DDS 減少率 |
| --------------- | --- | -------- | ------------ | ---------- |
| DDS             | 686 | 366.73   | 251,576      | –          |
| Zenoh           | 31  | 213.45   | 6,617        | 97.37%     |
| Zenoh + RG      | 13  | 136.54   | 1,775        | 99.29%     |
| Zenoh + WS      | 17  | 276.41   | 4,699        | 98.13%     |
| Zenoh + RG + WS | 1   | 82       | 82           | 99.97%     |

結論：搭配 **資源泛化** 與 **Warm Start**，Zenoh 幾乎徹底消除了 discovery storm，將 DDS 的開銷降低 **高達 99.97%**。

---

## 認識 Zenoh Router

雖然點對點模式適合本地環境，但在更大型的機器人與分散式系統中，常需要 **擴展性、跨協定橋接與資料儲存**。這正是 **Zenoh Router (`zenohd`)** 的用途。

它相當於一個資料樞紐，可以在不同網路間路由資料，甚至作為多種協定的橋接器。

---

### 快速開始

先下載原始碼並編譯：

```sh
git clone https://github.com/eclipse-zenoh/zenoh.git
cd zenoh
cargo build --all-targets
```

啟動 router：

```sh
./target/debug/zenohd
```

此時，一個 Zenoh router 已在本地運行。

---

### 插件簡介

Zenoh Router 的威力在於 **插件系統**。透過插件，Zenoh 能與 ROS、DDS、MQTT，甚至 Web 完整互通。

* **[zenoh-plugin-ros2dds](https://github.com/eclipse-zenoh/zenoh-plugin-ros2dds/)**
  橋接 ROS2/DDS 流量，大幅降低 discovery 開銷，並支援跨 WAN 傳輸。

* **[zenoh-plugin-dds](https://github.com/eclipse-zenoh/zenoh-plugin-dds/)**
  橋接標準 DDS 系統（基於[cyclors](https://github.com/ZettaScaleLabs/cyclors), Cyclone 的低階 Rust API）。適合混合式 DDS + Zenoh 環境。

* **[zenoh-plugin-mqtt](https://github.com/eclipse-zenoh/zenoh-plugin-mqtt/)**
  與 MQTT broker 或 client 互通，讓 ROS 或 DDS 資料能流向雲端 IoT 平台。

* **[zenoh-plugin-webserver](https://github.com/eclipse-zenoh/zenoh-plugin-webserver/)**
  以 HTTP REST 方式存取 Zenoh 資源，可透過 `curl` 直接操作。

* **[zenoh-plugin-ros1](https://github.com/eclipse-zenoh/zenoh-plugin-ros1/)**
  橋接 ROS1 topic，支援 **ROS1 + ROS2 + Zenoh-native** 混合部署。

---

### 插件組合應用場景

Zenoh 的優勢之一，就是能在同一個 router 中 **同時運行多個插件**：

1. **ROS2 在 WiFi 上的 discovery 減輕**
   使用 **zenoh-plugin-ros2dds**，可減少超過 99% DDS discovery 封包。

2. **ROS2 + MQTT 雲端儀表板**
   ROS2 → `ros2dds` → Zenoh → `mqtt` → 雲端平台 (Grafana/Node-RED)。

3. **ROS1 + ROS2 互通**
   同時啟用 `ros1` 與 `ros2dds` 插件，即可透明轉換，不必額外 ROS bridge 節點。

4. **DDS + Web 應用整合**
   DDS ↔ `dds` ↔ Zenoh ↔ `webserver` ↔ HTTP，用 `curl` 直接讀取工控系統資料。

5. **全能整合樞紐**
   一個 Zenoh router 可同時：

   * 服務 ROS1/ROS2
   * 連結 DDS 與 MQTT
   * 讓 Web 應用透過 REST 直接操作資料

Zenoh 將所有這些協定，統一在同一個 **資料經緯 (data fabric)**。

---

### 範例: zenoh-plugin-storage

Router 亦可用來保存資料。以下為記憶體儲存範例：

```sh
./target/debug/zenohd --cfg='plugins/storage_manager/storages/demo:{key_expr:"demo/example/**",volume:"memory"}'
```

互動方式：

```sh
./target/debug/examples/z_put
./target/debug/examples/z_get
```

或透過 HTTP：

```sh
curl -X PUT -d '"Hello World!"' http://localhost:8000/demo/example/test
curl http://localhost:8000/demo/example/test
```


借助 Zenoh Router能獲得一個 **高效能、避免 discovery storm、跨協定整合** 的資料骨幹，無論是機器人、IoT 還是分散式系統皆適用。

---

## 總結

使用 **Zenoh P2P 模式**，你能自動發現並高效進行本地通訊。
搭配 **Zenoh Router**，則進一步擴展：

* **網際網路規模的路由能力**
* **REST 整合**
* **資料持久化**

最重要的是：**再也沒有 discovery storm**。

Zenoh 提供的彈性，讓你能從一群透過 WiFi 溝通的機器人，平順擴展到雲端應用，而不再受 DDS 的高開銷或 MQTT 的 broker 限制。
