use anyhow::*;
use obniz_rust::*;
use std::env;
// cargo run --example test_crates 'YOUR-OBNIZ-ID'
fn main() -> anyhow::Result<()> {
    let obniz_id = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires obniz_id! as argument"));
    println!("{}", obniz_id);
    let mut obniz_object = obniz::connect(&obniz_id).context("connection failed ")?;
    //obniz_object.display_clear().context("clear is failed")?;
    obniz_object
        .display_text("work with crate")
        .context("dislay is failed")?;
    Ok(())
}
