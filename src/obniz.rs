use futures::stream::ForEach;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::*;

use std::net::TcpStream;

use anyhow::*;
use serde_json::Value;
use url::Url;

const OBNIZE_WEBSOKET_HOST: &str = "wss://obniz.io";
pub type ObnizWSocket = WebSocket<MaybeTlsStream<TcpStream>>;
pub type ReceiveForeach = ForEach<SplitStream<MaybeTlsStream<ObnizWSocket>>>;
///
/// Obniz
///
#[derive(Debug)]
pub struct Obniz {
    pub obniz_id: String,
    pub websocket: ObnizWSocket,
    pub api_url: Url,
    receive_foreeach: ReceiveForeach,
}

impl Obniz {
    async fn new(id: &str, socket: ObnizWSocket, api_url_: url::Url) -> Obniz {
        let id = id.to_string();
        // TODO 受信メッセージの
        let (write, read) = socket.split();
        let receive_thread = {
            read.for_each(|message| async {
                // let data = message.unwrap().into_data();
                // ここでメッセージの振り分けを行う
                // メッセージ送信　チャンネル使う
                //
                println!("receive message !!")
            })
        };
        Obniz {
            obniz_id: id,
            api_url: api_url_,
            websocket: socket,
            receive_foreeach: receive_thread,
        }
    }
}

pub fn connect(obniz_id: &str) -> anyhow::Result<Obniz> {
    let redirect_host = get_redirect_host(&(obniz_id.to_string()))?;
    let api_url = endpoint_url(&redirect_host, &obniz_id)?;
    let (ws_stream, _response) = tungstenite::connect_async(&api_url)
        .await
        .context("Failed to connect")?;

    Ok(Obniz::new(obniz_id, ws_stream, api_url))
}

fn endpoint_url(host: &str, obniz_id: &str) -> anyhow::Result<url::Url> {
    if !host.starts_with("wss://") {
        return Err(anyhow!("Illegal url, host needs to start with 'wss://'"));
    }

    let endpoint = format!("{}/obniz/{}/ws/1", host, obniz_id);
    url::Url::parse(&endpoint).context("Failed to parse endpoint url")
    // TODO add unit test
}

fn get_redirect_host(obniz_id: &String) -> anyhow::Result<String> {
    let url = endpoint_url(OBNIZE_WEBSOKET_HOST, obniz_id)?;
    //Websokcet接続
    let (mut ws_stream, _response) = tungstenite::connect(url).context("Failed to connect")?;

    let message = ws_stream.read_message().context("Fail to read message")?;
    //　接続するとリダイレクトアドレスが入ったjsonが返るのでパースする
    let message = message.to_text().context("fail to parse text")?;

    let res: Value = serde_json::from_str(message).context("Failed to parse json")?;
    let json_redirect_host = &res[0]["ws"]["redirect"];
    let redirect_host = match json_redirect_host.as_str() {
        // ダブルクォートが入るので除去するためにstrに一旦する
        Some(host) => host.to_string(),
        None => return Err(anyhow!("Failed to get redirect host name")),
    };
    println!("redirect_host : {redirect_host}");
    if redirect_host.is_empty() {
        return Err(anyhow!("Redirect host name is empty"));
    }
    if !redirect_host.starts_with("wss://") {
        return Err(anyhow!("Redirect host name is bad format"));
    }

    Ok(redirect_host)
}

// pub fn enable_reset_obniz_on_ws_disconnection(enable :bool){
//   let reset = request::WS{
//     reset_obniz_on_ws_disconnection: enable,
//   };
//   let req = vec![reset];
//   //ws_stream.write_message
// }

#[cfg(test)]
mod tests {
    // use super::*;
    // use anyhow::*;
    // use url::{Url,Host, ParseError};
    // #[test]
    // fn test_endpoint_url() {
    //   let result_url = endpoint_url("ws://test_host", "0000-0000")?;
    //   assert!(result_url.host() == Some(Host::Domain("test_host")));
    // }
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// ここ以降は再検討するので一旦コメントアウト

// #[derive(Debug)]
// pub struct Io {
// }

// pub enum OutputType{
//   PushPull5v,
//   PushPull3v,
//   OpenDrain,
// }

// pub enum  PullType{
//   PullUp5v,
//   PullUp3v,
//   PullDown,
//   Float,
// }

// pub trait Io {
// //   pub fn new(self,pin: u8) -> Result<Io> ;
//   fn get(&mut self,pin :u8)->anyhow::Result<bool>;
// //   pub fn set(self,value: bool);
// //   pub fn deinit(self);
// //   pub fn setAsInput(self,enable_stream_callback: bool);
// //   pub fn setAsOutput(self,value : bool);
// //   pub fn setOutputType(self,output_type : OutputType);
// //   pub fn setPullType(self,pull_type : PullType);
// }

// impl Io for Obniz{
// //   pub fn new(self,pin: u8) -> Io {
// //     unimplemented!();
// //     Io{}
// //   }

//   fn get(&mut self, pin : u8)->anyhow::Result<bool>{
//     // Json Example
//     // [
//     //   {
//     //       "io0": "get"
//     //   }
//     // ]
//     // ↓　↑　これを実現するシリアライザ・デシリアライザを実装した方が良いかも。
//     // IO{
//     //   pin: u8,
//     //   value: String ,// &str
//     // }

//     let io_key = format!("io{}",pin);
//     let json = serde_json::json!([{&io_key:"get"}]);

//     let json = serde_json::to_string(&json)?;
//     let msg = tungstenite::Message::from(json);
//     let _ = self.websocket.write_message(msg);
//     let message = self.websocket.read_message()?;
//     let message_str = message.to_text().context("fail to parse text")?;
//     let response : Vec<Response> = serde_json::from_str(message_str)?;
//     // レスポンスの切り分け必要ioxをまたどうにかしなきゃ行けない
//     // マクロ使う？

//   }
//   pub fn set(self,value: bool) {
//     unimplemented!();
//   }
//   pub fn deinit(self){
//     unimplemented!();
//   }
//   pub fn setAsInput(self,enable_stream_callback: bool){
//     // [
//     // {
//     //     "io0": {
//     //         "direction": "input",
//     //         "stream": false
//     //     }
//     // }
//     // ]
//     unimplemented!();
//   }

//   pub fn setAsOutput(self,value : bool){
//   //   [
//   //     {
//   //         "io0": {
//   //             "direction": "output",
//   //             "value": true
//   //         }
//   //     }
//   //   ]
//   }

//   pub fn setOutputType(self,output_type : OutputType){
//     // [
//     // {
//     //     "io0": {
//     //         "output_type": "push-pull5v"
//     //     }
//     // }
//     // ]

//     unimplemented!();
//   }

//   pub fn setPullType(self,pull_type : PullType){
//   //   [
//   //     {
//   //         "io0": {
//   //             "pull_type": "pull-up5v"
//   //         }
//   //     }
//   //   ]
//       unimplemented!();
//   }

//}

// pub struct IoAnimation {
//   // websocket 持たす?
// }

// impl IoAnimation {

//   pub fn new()-> IoAnimation{
//     unimplemented!();
//     IoAnimation{}
//   }

//   pub fn init_animation(){

//     unimplemented!();
//   }

//   pub fn change_state(){
//     // [
//     // {
//     //     "io": {
//     //         "animation": {
//     //             "name": "animation-1",
//     //             "status": "pause"
//     //         }
//     //     }
//     // }
//     // ]
//     unimplemented!();
//   }
// }

// pub struct AD {
//   pin: u8,
// }

// impl AD {
//   pub fn new(pin: u8) -> AD {
//     AD {
//       pin: pin
//     }
//   }

//   pub fn get(self) -> f64 {
//     unimplemented!();
//     0.0
//   }

//   pub fn deinit(self) {
//     // [
//     // {
//     //     "ad0": null
//     // }
//     // ]
//     unimplemented!();
//   }
// }

// pub struct PWM {
//   pwm_number: u8,
// }

// impl PWM {
//   pub fn new(pwm_number: u8) -> PWM {
//     unimplemented!();
//     PWM {
//       pwm_number: pwm_number
//     }
//   }

//   ///
//   /// init pwm module
//   /// pin 0-5
//   ///
//   pub fn init(self, pin : u8) {
//     unimplemented!();
//   }

//   ///
//   /// unit : Hz
//   /// 1 ≤ value ≤ 80000000
//   pub fn setFrequency(self, freq : u64) {
//     unimplemented!();
//   }

//   pub fn setPulseWidth(self, width_msec : u64) {
//     unimplemented!();
//   }

//   pub fn modulate(self, symbol_length: f64, bitArray: Vec<bool>){
//     // bitArray needs to be like [0, 1, 1, 0, 0, 1, 1, 0]
//     unimplemented!();
//   }

//   pub fn deinit(self) {
//     unimplemented!();
//   }

// }

// pub struct Uart{

// }

// impl Uart {
//   pub fn new() -> Uart {
//     unimplemented!();

//     Uart{}
//   }

//   pub fn init() {
//     unimplemented!();

//   }

//   pub fn send(data : Vec<u8>){
//     unimplemented!();

//   }

//   pub fn deinit() {
//     unimplemented!();
//   }
//   pub fn set_receive_callback(){
//     unimplemented!();
//   }
// }

// pub struct Spi {

// }

// impl Spi {

//   pub fn new() -> Spi {
//     unimplemented!();
//     Spi{}
//   }

//   pub fn init_as_master() {
//     unimplemented!();
//   }

//   pub fn deinit() {
//     unimplemented!();

//   }

//   pub fn write(data : Vec<u8>) {
//     unimplemented!();
//   }
//   pub fn write_with_callback(data : Vec<u8>
//     //read call back
//   ) {
//     unimplemented!();
//   }

//   pub fn set_read_callback(){
//     unimplemented!();
//   }
// }

// struct I2c {

// }

// impl I2c {
//   pub fn new() -> I2c {
//     unimplemented!();
//     I2c{}
//   }
//   pub fn init_as_master() {unimplemented!();}
//   pub fn init_as_slave() {
//     unimplemented!();
//   }
//   pub fn write(address: u16 ,
//     address_bits : u8, //default 7
//     data : Vec<u8>) {
//     unimplemented!();
//   }
//   pub fn write_with_callback(address: u16 ,
//     address_bits : u8, //default 7
//     data : Vec<u8>
//     //read call back
//   ) {
//     unimplemented!();
//   }

//   pub fn set_read_callback(){
//     unimplemented!();
//   }
// }

// pub struct LogicAnalyzer{}

// impl LogicAnalyzer {
//   pub fn init() {
//     unimplemented!();
//   }

//   pub fn deinit(){
//     unimplemented!();
//   }

//   pub fn set_data_response_callback(){
//     unimplemented!();
//   }
// }

// pub struct Measurement {}

// impl Measurement {
//   /// set callback
//   pub fn echo() {
//     unimplemented!();
//   }
// }

// pub fn set_read_callback(){
//     unimplemented!();
//   }
// }

// struct I2c {

// }

// impl I2c {
//   pub fn new() -> I2c {
//     unimplemented!();
//     I2c{}
//   }
//   pub fn init_as_master() {unimplemented!();}
//   pub fn init_as_slave() {
//     unimplemented!();
//   }
//   pub fn write(address: u16 ,
//     address_bits : u8, //default 7
//     data : Vec<u8>) {
//     unimplemented!();
//   }
//   pub fn write_with_callback(address: u16 ,
//     address_bits : u8, //default 7
//     data : Vec<u8>
//     //read call back
//   ) {
//     unimplemented!();
//   }

//   pub fn set_read_callback(){
//     unimplemented!();
//   }
// }

// pub struct LogicAnalyzer{}

// impl LogicAnalyzer {
//   pub fn init() {
//     unimplemented!();
//   }

//   pub fn deinit(){
//     unimplemented!();
//   }

//   pub fn set_data_response_callback(){
//     unimplemented!();
//   }
// }

// pub struct Measurement {}

// impl Measurement {
//   /// set callback
//   pub fn echo() {
//     unimplemented!();
//   }
// }

/* */

pub enum QrCorrectionType {
    L,
    M,
    Q,
    H,
}
pub enum DisplayRawCollorDepth {
    OneBit,
    FourBit,
    SixteenBit,
}
/// TODOこういう実装をすれば良いはず
pub trait ObnizDisplay {
    fn display_text(&mut self, text: &str) -> anyhow::Result<()>;
    fn display_clear(&mut self) -> anyhow::Result<()>;
    // fn qr(text : &str , correction_type : QrCorrectionType );
    // fn raw(raw : Vec<u16> , color_depth: DisplayRawCollorDepth );
    // fn pin_assign(pin: u8 , module_name :&str, pin_name :&str);
}

impl ObnizDisplay for Obniz {
    fn display_text(&mut self, text: &str) -> anyhow::Result<()> {
        let json = serde_json::json!([{"display":{"text":text}}]).to_string();
        let msg = tungstenite::Message::from(json);
        self.websocket.write_message(msg).context("test")
    }

    fn display_clear(&mut self) -> anyhow::Result<()> {
        let json = serde_json::json!([{"display":{"clear":true}}]).to_string();
        let msg = tungstenite::Message::from(json);
        self.websocket.write_message(msg).context("test")
    }

    //   // pub fn qr(text : &str , correction_type : QrCorrectionType ){
    //   unimplemented!();
    // }
    // pub fn raw(raw : Vec<u16> , color_depth: DisplayRawCollorDepth ){
    //   unimplemented!();
    // }

    // pub fn pin_assign(pin: u8 , module_name :&str, pin_name :&str){
    //   unimplemented!();
    // }
}

// pub struct Switch {

// }

// pub enum SwitchState {
//   None,Push,Left,Right,
// }

// impl Switch {
//   pub fn get() -> SwitchState {
//     unimplemented!();
//   }
// }

// pub struct TCP {

// }

// impl TCP {
//   pub fn connect(port:u16, domain: &str) {
//     unimplemented!();
//   }

//   pub fn disconnect(){
//     unimplemented!();
//   }
//   pub fn write(data : Vec<u8>) {
//     unimplemented!();
//   }
//   pub fn write_with_callback(data : Vec<u8>
//     //read call back
//   ) {
//     unimplemented!();
//   }

//   pub fn set_read_callback(){
//     unimplemented!();
//   }
// }

// pub struct Wifi {}

// impl Wifi {
//   pub fn scan() -> Vec<u8> {
//     unimplemented!();
//   }
// }

// pub struct BleHci{

// }

// impl BleHci {
//   pub fn init(){
//     unimplemented!();
//   }

//   pub fn deinit(){
//     unimplemented!();
//   }

//   pub fn write(data : Vec<u8>) {
//     unimplemented!();
//   }
//   pub fn write_with_callback(data : Vec<u8>
//     //read call back
//   ) {
//     unimplemented!();
//   }

//   pub fn set_read_callback(){
//     unimplemented!();
//   }

//   pub fn advertisement_filter(
//     // TODO 引数考える
//   ){
//     unimplemented!();
//   }
// }

// struct Message{}
// impl Message {

//   pub fn send(data: &str, to: Vec<String> ){
//     unimplemented!();
//   }

//   pub fn set_receive_callback(
//     // TODO 引数考える
//   ){
//     unimplemented!();
//   }

// }

// struct Plugin {}
// impl Plugin {

//   pub fn send(data: Vec<u8> ){
//     unimplemented!();
//   }

//   pub fn set_receive_callback(
//     // TODO 引数考える
//   ){
//     unimplemented!();
//   }

// }

// // debug は
