use std::env;
use tungstenite::{connect,Message};
use serde_json::{Value};

const OBNIZE_WEBSOKET_HOST:&str = "wss://obniz.io"; // FIXME wss://obniz.ioだとトラブル発生　なぜ？

fn obniz_endpoint_url(host : &str, obniz_id: &String) -> url::Url {
    let endpoint = format!("{}/obniz/{}/ws/1",host,obniz_id);
    println!("{}",endpoint);
    url::Url::parse(&endpoint).unwrap()
}

fn get_obniz_redirect_host(obniz_id :&String) -> String {
    let url = obniz_endpoint_url(OBNIZE_WEBSOKET_HOST,obniz_id);
    //Websokcet接続
    let ( mut ws_stream, _response) = connect(url).expect("Failed to connect");
    let message = ws_stream.read_message().expect("Fail to read message");
    let message = message.to_text().expect("fail to parse text");
    println!("{}", message);
    let v: Value = serde_json::from_str(message).expect("Failed to parse json");
    let host = v[0]["ws"]["redirect"].as_str().unwrap();
    host.to_string()
}

fn main() {
    let obniz_id =
        env::args().nth(1).unwrap_or_else(|| panic!("this program requires obniz_id! as argument"));
    let redirect_host = get_obniz_redirect_host(&obniz_id);
    println!("redirect host = {}", redirect_host);

    let url = obniz_endpoint_url(redirect_host.as_str(),&obniz_id);

    let ( mut ws_stream, _response) = connect(url).expect("Failed to connect");
    let json = serde_json::json!([{"display":{"clear":true}}, {"display":{"text":"Works fine."}}]);
    let message = Message::text(json.to_string());
    ws_stream.write_message(message).expect("Fail to write_message");
    
}
