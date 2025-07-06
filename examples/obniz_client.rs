use serde_json::Value;
use std::env;
use tungstenite::connect;
use tungstenite::Message;

const OBNIZE_WEBSOKET_HOST: &str = "wss://obniz.io";
fn obniz_endpoint_url(host: &str, obniz_id: &String) -> url::Url {
    let endpoint = format!("{host}/obniz/{obniz_id}/ws/1");
    println!("{endpoint}");
    url::Url::parse(&endpoint).unwrap()
}

fn get_obniz_redirect_host(obniz_id: &String) -> String {
    let url = obniz_endpoint_url(OBNIZE_WEBSOKET_HOST, obniz_id);
    //Websokcet接続
    let (mut ws_stream, _response) = connect(url.as_str()).expect("Failed to connect");
    let message = ws_stream.read().expect("Fail to read message");
    let message = message.to_text().expect("fail to parse text");
    println!("message {message}");
    let v: Value = serde_json::from_str(message).expect("Failed to parse json");
    let host = v[0]["ws"]["redirect"].as_str().unwrap();
    host.to_string()
}

fn main() {
    let obniz_id = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires obniz_id! as argument"));
    let redirect_host = get_obniz_redirect_host(&obniz_id);
    println!("redirect host = {redirect_host}");

    let url = obniz_endpoint_url(redirect_host.as_str(), &obniz_id);
    println!("***connect !!***");
    let (mut ws_stream, _response) = connect(url.as_str()).expect("Failed to connect");
    let welcome_message = ws_stream.read().expect("Failed to read message");
    let welcome_message = welcome_message.to_text().expect("Failed to parse text ");
    println!("**welcome message *** \n{welcome_message} \n***********");

    let json =
        serde_json::json!([{"display":{"clear":true}}, {"display":{"text":"Works fine...."}}]);
    let message = Message::text(json.to_string());
    ws_stream.send(message).expect("Fail to send message");
}

// How to run the
// cargo run --example obniz_client XXXX-XXXX
