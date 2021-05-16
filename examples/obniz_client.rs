use std::env;
use tungstenite::{connect};
use serde_json::{Value};

const OBNIZE_WEBSOKET_HOST:&str = "wss://obniz.io"; // FIXME wss://obniz.ioだとトラブル発生　なぜ？

#[tokio::main]
async fn main() {
    let obniz_id =
        env::args().nth(1).unwrap_or_else(|| panic!("this program requires obniz_id! as argument"));
    let obniz_endpoint = format!("{}/obniz/{}/ws/1",OBNIZE_WEBSOKET_HOST,obniz_id);
    let url = url::Url::parse(&obniz_endpoint).unwrap();
    println!("url {}",url);

    //Websokcet接続
    let ( mut ws_stream, _response) = connect(url).expect("Failed to connect");
    // let (ws_stream, response) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");
    let message = ws_stream.read_message().expect("Fail to read message");
    let message = message.to_text().expect("fail to parse text");
    //println!("{}", message);
    let v: Value = serde_json::from_str(message).expect("Failed to parse json");
    println!("{}", v);
    let redirect_host = &v[0]["ws"]["redirect"];
    println!("{}", redirect_host);
}
