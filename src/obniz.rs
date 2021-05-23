// use tungstenite::{connect};
//  use tungstenite::Message;
// use super::api::response::*;
// use serde_json::{Value};

// const OBNIZE_WEBSOKET_HOST:&str = "wss://obniz.io";

#[derive(Debug)]
pub struct Obniz{
  obniz_id: String,
  is_connected: bool,
}


impl Obniz{
  pub fn new(obniz_id_: &str) -> Obniz{
    Obniz{
      obniz_id: obniz_id_.to_string(),
      is_connected: false,
    }
  }

  // pub fn connect(host: &str, obniz_id: &str){

  // }

  pub fn is_connected() -> bool {
    false
  }

  // fn endpoint_url_with_host(host : &str, obniz_id: &str) -> url::Url {
  //   let endpoint = format!("{}/obniz/{}/ws/1",host,obniz_id);
  //   println!("{}",endpoint);
  //   url::Url::parse(&endpoint).unwrap()
  // }


  // fn endpoint_url(obniz_id : &String)-> url::Url {
  //   Obniz::endpoint_url_with_host(OBNIZE_WEBSOKET_HOST,obniz_id)
  // }


  // fn get_obniz_redirect_host(obniz_id :&String) -> String {
  //   let url = Obniz::endpoint_url(obniz_id);
  //   //Websokcet接続
  //   let ( mut ws_stream, _response) = connect(url).expect("Failed to connect");
  //   let message = ws_stream.read_message().expect("Fail to read message");
  //   let message = message.to_text().expect("fail to parse text");
  //   println!("{}", message);
  //   let v: Value = serde_json::from_str(message).expect("Failed to parse json");
  //   let host = v[0]["ws"]["redirect"].as_str().unwrap();
  //   host.to_string()
  // }

}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }


}
