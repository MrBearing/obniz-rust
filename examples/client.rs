use std::env;

use futures_util::{future, pin_mut, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tungstenite::{connect};

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
    // redirect先のホスト名が返ってくるので、ホスト名を変えて再度アクセス
    let message = ws_stream.read_message().expect("Fail to read message");
    println!("{}", message);
}
