use std::env;
use obniz_rust::*;
use async_std::task::*;
fn main() {
    let obniz_id = env::args().nth(1).unwrap_or_else(|| panic!("this program requires obniz_id! as argument"));

    let obniz_object = block_on(obniz::connect(&obniz_id));
    match &obniz_object {
        Ok(it) => println!("api_url : {}",it.api_url),
        Err(error) => println!("{}",error),
    }
}
