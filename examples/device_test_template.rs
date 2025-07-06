use obniz_rust::*;
use std::time::Duration;
use tokio::time::{sleep, timeout};

// IMPORTANT: Replace "YOUR-OBNIZ-ID" with your actual obniz device ID
const OBNIZ_ID: &str = "YOUR-OBNIZ-ID"; // Example: "1234-5678"

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Obnizデバイステスト");
    println!("📱 デバイスID: {}", OBNIZ_ID);
    
    if OBNIZ_ID == "YOUR-OBNIZ-ID" {
        println!("❌ エラー: 実際のObnizデバイスIDを設定してください");
        println!("💡 OBNIZ_ID定数を編集してから実行してください");
        return Err("Device ID not configured".into());
    }
    
    // obnizデバイスに接続
    println!("\n🔌 デバイスに接続中...");
    let obniz = match timeout(Duration::from_secs(30), connect_async(OBNIZ_ID)).await {
        Ok(result) => match result {
            Ok(device) => {
                println!("✅ 接続成功!");
                device
            }
            Err(e) => {
                println!("❌ 接続エラー: {:?}", e);
                return Err(e.into());
            }
        },
        Err(_) => {
            println!("❌ 接続タイムアウト");
            return Err("Connection timeout".into());
        }
    };

    // ディスプレイテスト
    println!("\n🖥️ ディスプレイテスト...");
    let display = obniz.display();
    
    match display.clear().await {
        Ok(_) => println!("✅ 画面クリア成功"),
        Err(e) => println!("❌ 画面クリアエラー: {:?}", e),
    }

    sleep(Duration::from_millis(500)).await;

    match display.text("Hello Rust!").await {
        Ok(_) => println!("✅ テキスト表示成功"),
        Err(e) => println!("❌ テキスト表示エラー: {:?}", e),
    }

    sleep(Duration::from_secs(2)).await;

    // IOテスト（ピン0を使用）
    println!("\n📌 IOテスト (ピン0)...");
    let io = obniz.io();
    
    // LED点滅テスト
    for i in 1..=3 {
        println!("  {}回目の点滅", i);
        
        match io.set_pin_as_output(0, true).await {
            Ok(_) => println!("    ✅ ピン0 ON"),
            Err(e) => println!("    ❌ ピン0 ON エラー: {:?}", e),
        }
        
        sleep(Duration::from_millis(500)).await;
        
        match io.set_pin_as_output(0, false).await {
            Ok(_) => println!("    ✅ ピン0 OFF"),
            Err(e) => println!("    ❌ ピン0 OFF エラー: {:?}", e),
        }
        
        sleep(Duration::from_millis(500)).await;
    }

    // 入力テスト
    println!("\n📥 入力テスト (ピン1)...");
    match io.set_pin_as_input(1, false).await {
        Ok(_) => {
            println!("✅ ピン1入力設定成功");
            
            match io.get_pin(1).await {
                Ok(state) => println!("✅ ピン1状態: {}", state),
                Err(e) => println!("❌ ピン1読み取りエラー: {:?}", e),
            }
        }
        Err(e) => println!("❌ ピン1入力設定エラー: {:?}", e),
    }

    // PWMテスト
    println!("\n⚡ PWMテスト...");
    let pwm = obniz.pwm();
    
    match pwm.configure_channel(0, PwmConfig {
        io_pin: 2,
        frequency: 1000,
        pulse_width_ms: 0.5,
    }).await {
        Ok(_) => {
            println!("✅ PWM設定成功 (ピン2, 1kHz, 50%)");
            sleep(Duration::from_secs(2)).await;
            
            match pwm.deinit_channel(0).await {
                Ok(_) => println!("✅ PWM停止"),
                Err(e) => println!("❌ PWM停止エラー: {:?}", e),
            }
        }
        Err(e) => println!("❌ PWM設定エラー: {:?}", e),
    }

    // スイッチテスト
    println!("\n🔘 スイッチテスト...");
    let switch = obniz.switch();
    
    match switch.get_state().await {
        Ok(state) => println!("✅ スイッチ状態: {}", state),
        Err(e) => println!("❌ スイッチエラー: {:?}", e),
    }

    // 最終メッセージ
    println!("\n📝 最終メッセージ...");
    match display.text("Test Complete!").await {
        Ok(_) => println!("✅ 最終メッセージ表示成功"),
        Err(e) => println!("❌ 最終メッセージエラー: {:?}", e),
    }

    println!("\n🎉 テスト完了!");

    Ok(())
}

// 使用方法:
// 1. OBNIZ_ID定数を実際のデバイスIDに変更
// 2. cargo run --example device_test_template