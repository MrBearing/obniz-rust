use anyhow::*;
use obniz_rust::*;
use std::env;

// cargo run --example test_crates 'YOUR-OBNIZ-ID'
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let obniz_id = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires obniz_id! as argument"));
    println!("Connecting to obniz: {}", obniz_id);

    let obniz = connect_async(&obniz_id)
        .await
        .context("connection failed")?;
    let display = obniz.display();

    display.clear().await.context("clear failed")?;
    display
        .text("work with crate")
        .await
        .context("display failed")?;

    println!("Test completed successfully!");
    Ok(())
}
