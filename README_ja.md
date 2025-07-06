# Obniz Rust

[![CI](https://github.com/MrBearing/obniz-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/MrBearing/obniz-rust/actions/workflows/ci.yml)

🦀 WebSocket通信によるobniz IoTデバイス制御のための包括的なRustライブラリ

## 特徴

- **Async/Await対応**: tokioを使用した完全非同期API
- **型安全性**: カスタムエラーハンドリングによる強力な型付け
- **包括的カバレッジ**: すべての主要なobniz機能をサポート
- **コールバックシステム**: リアルタイムイベント処理とコールバック
- **メモリ安全**: Rustのメモリ安全保証で構築

## サポート対象モジュール

| モジュール | 状態 | 説明 |
|-----------|------|------|
| 🔧 **IO** | ✅ 完成 | デジタルピン制御、ストリームモード、コールバック |
| 📺 **Display** | ✅ 完成 | テキスト、グラフィック、QRコード、生ピクセルデータ |
| ⚡ **AD** | ✅ 完成 | アナログ-デジタル変換、電圧読み取り |
| 🌊 **PWM** | ✅ 完成 | PWM生成、サーボ制御、変調 |
| 📡 **UART** | ✅ 完成 | フロー制御付きシリアル通信 |
| 🔘 **Switch** | ✅ 完成 | 内蔵スイッチ状態監視 |
| ⚙️ **System** | ✅ 完成 | デバイス制御、リセット、設定 |
| 🔌 **SPI** | 🔄 予定 | SPIマスター/スレーブ通信 |
| 📶 **I2C** | 🔄 予定 | I2Cバス通信 |
| 📊 **Logic Analyzer** | 🔄 予定 | デジタル信号解析 |
| 📏 **Measurement** | 🔄 予定 | 高度な測定機能 |

## クイックスタート

`Cargo.toml`に追加：

```toml
[dependencies]
obniz-rust = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 基本的な使用方法

```rust
use obniz_rust::*;

#[tokio::main]
async fn main() -> ObnizResult<()> {
    // obnizデバイスに接続
    let obniz = connect_async("1234-5678").await?;
    
    // デジタルIO
    let io = obniz.io();
    io.set_pin(0, true).await?;  // ピン0をHighに設定
    let state = io.get_pin(1).await?;  // ピン1を読み取り
    
    // ディスプレイ
    let display = obniz.display();
    display.text("Hello, Rust!").await?;
    display.qr("https://obniz.io", QrCorrectionType::Medium).await?;
    
    // アナログ読み取り
    let ad = obniz.ad();
    let voltage = ad.get_voltage(0).await?;
    println!("電圧: {:.2}V", voltage);
    
    // LED制御用PWM
    let pwm = obniz.pwm();
    pwm.square_wave(0, 2, 1000).await?;  // ピン2で1kHz
    
    Ok(())
}
```

### コールバック使用の高度な例

```rust
use obniz_rust::*;

#[tokio::main]
async fn main() -> ObnizResult<()> {
    let obniz = connect_async("1234-5678").await?;
    
    // リアルタイムピン監視用ストリームモード
    let io = obniz.io();
    io.set_pin_callback(0, |state| {
        println!("ピン0が変更されました: {}", state);
    }).await?;
    
    // アナログセンサー監視
    let ad = obniz.ad();
    ad.set_channel_callback(0, |voltage| {
        println!("センサー値: {:.2}V", voltage);
    }).await?;
    
    // スイッチイベント処理
    let switch = obniz.switch();
    switch.on_push(|| {
        println!("スイッチが押されました！");
    }).await?;
    
    // コールバック受信のためプログラムを実行し続ける
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    
    Ok(())
}
```

## モジュールドキュメント

### IO制御

```rust
let io = obniz.io();

// 基本的なピン操作
io.set_pin(0, true).await?;           // ピンをHighに設定
io.set_pin_as_output(1, false).await?; // 出力として設定
io.set_pin_as_input(2, true).await?;   // ストリーム付き入力として設定

// 高度な設定
let config = IoConfig {
    direction: Direction::Output,
    value: Some(true),
    output_type: Some(OutputType::PushPull5v),
    pull_type: Some(PullType::PullUp5v),
    stream: Some(false),
};
io.configure_pin(3, config).await?;

// リアルタイム監視
io.set_pin_callback(0, |state| {
    println!("ピン状態が変更されました: {}", state);
}).await?;
```

### ディスプレイ制御

```rust
let display = obniz.display();

// テキストと基本操作
display.text("Hello World").await?;
display.clear().await?;
display.brightness(75).await?;

// グラフィック
display.rect(10, 10, 50, 30, true, true).await?;  // 塗りつぶし矩形
display.circle(80, 40, 20, false, true).await?;   // 円の輪郭
display.line(0, 0, 127, 63, true).await?;         // 対角線

// QRコード
display.qr("https://obniz.io", QrCorrectionType::High).await?;

// 生ピクセルデータ
let config = RawDisplayConfig {
    width: 128,
    height: 64,
    color_depth: DisplayRawColorDepth::OneBit,
    data: pixel_data,
};
display.raw(config).await?;
```

### アナログ入力 (AD)

```rust
let ad = obniz.ad();

// 単一読み取り
let voltage = ad.get_voltage(0).await?;
println!("電圧: {:.2}V", voltage);

// 複数チャンネル
let readings = ad.get_voltages(vec![0, 1, 2]).await?;
for reading in readings {
    println!("AD{}: {:.2}V", reading.channel, reading.voltage);
}

// コールバック付きストリームモード
ad.set_channel_callback(0, |voltage| {
    let percentage = AdManager::voltage_to_percentage(voltage);
    println!("センサー: {:.2}V ({:.1}%)", voltage, percentage);
}).await?;

// ユーティリティ関数
let percentage = AdManager::voltage_to_percentage(3.3); // 66%
let is_safe = AdManager::is_voltage_safe(4.8); // true
```

### PWM生成

```rust
let pwm = obniz.pwm();

// 基本的なPWM
pwm.configure_channel(0, PwmConfig {
    io_pin: 5,
    frequency: 1000,
    pulse_width_ms: 0.5,
}).await?;

// サーボ制御 (0-180度)
pwm.servo(1, 6, 90.0).await?;

// 矩形波生成
pwm.square_wave(2, 7, 2000).await?; // ピン7で2kHz

// デューティサイクル制御
pwm.set_channel_duty_cycle(0, 1000, 25.0).await?; // 25%デューティ

// 通信用変調
let mod_config = ModulationConfig {
    modulation_type: ModulationType::Am,
    symbol_length_ms: 100.0,
    data: vec![0, 1, 1, 0, 1],
};
pwm.channel(0)?.modulate(mod_config).await?;
```

### UART通信

```rust
let uart = obniz.uart();

// 簡単な設定
let config = UartManager::simple_config(0, 1, 115200); // RX, TX, ボーレート
uart.init_channel(0, config).await?;

// データ送信
uart.send_string(0, "Hello UART!").await?;
uart.send_data(0, vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]).await?;

// 受信コールバック
uart.set_string_callback(0, |data| {
    println!("受信: {}", data);
}).await?;

// フロー制御付き高度な設定
let advanced_config = UartManager::flow_control_config(0, 1, 2, 3, 9600);
uart.init_channel(0, advanced_config).await?;
```

### スイッチ監視

```rust
let switch = obniz.switch();

// 現在の状態
let state = switch.get_state().await?;
println!("スイッチ: {}", state);

// イベントコールバック
switch.on_push(|| println!("押されました！")).await?;
switch.on_left(|| println!("左！")).await?;
switch.on_right(|| println!("右！")).await?;
switch.on_release(|| println!("離されました！")).await?;

// イベントのブロッキング待機
let pressed_state = switch.wait_for_press(Some(5000)).await?; // 5秒タイムアウト
switch.wait_for_release(None).await?; // タイムアウトなし
```

## エラーハンドリング

ライブラリは`ObnizResult<T>`型による包括的なエラーハンドリングを提供します：

```rust
match obniz.io().get_pin(0).await {
    Ok(state) => println!("ピン状態: {}", state),
    Err(ObnizError::InvalidPin(pin)) => println!("無効なピン: {}", pin),
    Err(ObnizError::Connection(msg)) => println!("接続エラー: {}", msg),
    Err(ObnizError::Timeout) => println!("操作がタイムアウトしました"),
    Err(e) => println!("その他のエラー: {}", e),
}
```

## テスト

### ユニットテスト

```bash
cargo test
```

### モックシステムを使用したテスト

実際のハードウェアなしでテストを行うための包括的なモックシステムが含まれています：

```rust
use obniz_rust::mock::*;

#[tokio::test]
async fn test_with_mock() {
    let mock_config = MockConfig::default();
    let mock_device = MockObniz::new(mock_config);
    let server = mock_device.server();
    
    // カスタムレスポンスの設定
    server.add_response("io0", responses::io_pin_state(0, true));
    
    // テスト実行
    let request = json!([{"io0": "get"}]);
    let response = server.process_message(Message::from(request.to_string())).await?;
    
    assert_eq!(response, Some(responses::io_pin_state(0, true)));
}
```

### モック統合例の実行

```bash
cargo run --example mock_integration_example
```

## 例

リポジトリには包括的な例が含まれています：

- **`basic_example.rs`** - 単純なIOとディスプレイ操作
- **`io_example.rs`** - 完全なIO機能のデモンストレーション
- **`display_example.rs`** - ディスプレイとグラフィック機能
- **`comprehensive_example.rs`** - すべてのモジュールが連携して動作
- **`mock_integration_example.rs`** - モックシステムの完全なデモンストレーション

例の実行：

```bash
cargo run --example comprehensive_example
```

### 実デバイスでのテスト

実際のobnizデバイスでテストする場合：

1. `device_test_template.rs`を新しいファイルにコピー
2. `"YOUR-OBNIZ-ID"`を実際のデバイスIDに変更
3. **重要**: 実際のデバイスIDを含むファイルをバージョン管理にコミットしないでください

```bash
# テンプレートをコピーしてデバイスIDを編集
cp examples/device_test_template.rs examples/my_device_test.rs
# my_device_test.rs内のOBNIZ_ID定数を編集
cargo run --example my_device_test
```

## テスト範囲

- **62の包括的なテスト** すべてのモジュールをカバー
- **モックWebSocketサーバー** ハードウェアなしでのテスト
- **統合テスト** 実際のシナリオでの動作確認
- **シリアライゼーション/デシリアライゼーション** テスト
- **エラーハンドリング** 包括的なエラーシナリオ
- **ユーティリティ関数** 数学的計算と変換

## デバイス互換性

テスト済みデバイス：
- obniz Board
- obniz Board 1Y
- M5StickC (obnizファームウェア)

## 依存関係

- `tokio` - 非同期ランタイム
- `tungstenite` - WebSocket通信
- `serde` - JSON シリアライゼーション
- `anyhow` - エラーハンドリング
- `futures` - 非同期ユーティリティ

## 開発者向け情報

### プロジェクト構造

```
src/
├── lib.rs          # ライブラリエントリポイント
├── obniz.rs        # コア接続とWebSocket処理
├── io.rs           # IOピン制御
├── display.rs      # ディスプレイ制御
├── ad.rs           # アナログ入力
├── pwm.rs          # PWM生成
├── uart.rs         # UART通信
├── switch.rs       # スイッチ監視
├── system.rs       # システム制御
├── error.rs        # エラー定義
└── mock.rs         # テスト用モックシステム
```

### 設計パターン

- **Manager パターン**: 各モジュールごとのマネージャー
- **Builder パターン**: 設定オブジェクトの構築
- **Callback システム**: イベント駆動型プログラミング
- **型安全性**: コンパイル時エラー検出
- **非同期設計**: パフォーマンス向上

## 貢献

貢献を歓迎します！IssuesやPull Requestsをお気軽に提出してください。

### 開発環境のセットアップ

```bash
git clone https://github.com/your-username/obniz-rust.git
cd obniz-rust
cargo build
cargo test
```

### 貢献ガイドライン

1. コードは`rustfmt`でフォーマットしてください
2. すべてのテストがパスすることを確認してください
3. 新機能にはテストを追加してください
4. ドキュメントを更新してください

## ライセンス

このプロジェクトはMITライセンスの下でライセンスされています - 詳細はLICENSEファイルを参照してください。

## 謝辞

- 素晴らしいIoTプラットフォームを提供する[obniz](https://obniz.io)
- 驚異的な非同期エコシステムを持つRustコミュニティ
- すべての貢献者とテスター

---

## 追加リソース

- [obniz公式ドキュメント](https://obniz.io/doc)
- [WebSocket APIリファレンス](https://obniz.io/doc/reference/websocket)
- [Rustの非同期プログラミング](https://rust-lang.github.io/async-book/)

開発やテストに関するご質問がありましたら、お気軽にIssueを作成してください。