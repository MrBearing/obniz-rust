use std::env;
use obniz_rust::*;
use anyhow::*;
fn main() -> anyhow::Result<()>{
    let obniz_id = env::args().nth(1).unwrap_or_else(|| panic!("this program requires obniz_id! as argument"));

    let mut obniz_object = obniz::connect(&obniz_id).context("connection failed ")?;
    obniz_object.clear().context("clear is failed")?;
    obniz_object.display_text("work with crate").context("dislay is failed")?;
    Ok(())
}