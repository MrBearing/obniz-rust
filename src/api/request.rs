use serde::{Serialize};


// root
#[derive(Debug,Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Request {
  Ws(WS),
  System(System),
  // IO
  Io0(Io),
  Io1(Io),
  Io2(Io),
  Io3(Io),
  Io4(Io),
  Io5(Io),
  Io6(Io),
  Io7(Io),
  Io8(Io),
  Io9(Io),
  Io10(Io),
  Io11(Io),

  // Io Animation
  Io{animation: IoAnimation},
  // AD
  Ad0(Ad),
  Ad1(Ad),
  Ad2(Ad),
  Ad3(Ad),
  Ad4(Ad),
  Ad5(Ad),
  Ad6(Ad),
  Ad7(Ad),
  Ad8(Ad),
  Ad9(Ad),
  Ad10(Ad),
  Ad11(Ad),
  // pwm
  Pwm0(Pwm),
  Pwm1(Pwm),
  Pwm2(Pwm),
  Pwm3(Pwm),
  Pwm4(Pwm),
  Pwm5(Pwm),
  // Uart
  Uart0(Uart),
  Uart1(Uart),

  Spi0(Spi),
  Spi1(Spi),
  I2c0(I2c),
  LogicAnalyzer(LogicAnalyzer),
  Switch(String),
  Mesure(Measure),
  Display(Display),
  Tcp0(Tcp),
  Wifi(Wifi),
  Ble(Ble),
  Message(Message),
  Plugin(Plugin),
}

#[derive(Debug,Serialize)]
#[serde(rename_all = "snake_case")]
pub struct WS {
  pub reset_obniz_on_ws_disconnection :bool,
}

#[derive(Debug,Serialize)]
#[serde(rename_all = "snake_case")]
pub enum System {
  Wait(i64),
  Reset(bool),
  Reboot(bool),
  SelfCheck(bool),
  KeepWorkingAtOffline(bool),
  Ping(bool),
  SleepSeconds(i64),
  SleepMinute(i64),
  SleepIoTrigger(bool),
}

#[derive(Debug,Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Io{
  String,
  InputDetail,
  bool,
  OutputDetail,
  Direction(String),
  Value(bool),
  OutputType(String),
  PullType(String),
  None,
}

#[derive(Debug,Serialize)]
#[serde(rename_all = "snake_case")]
pub struct InputDetail {
  pub direction : String,
  pub stream : bool,
}

#[derive(Debug,Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OutputDetail{
  pub direction: String,
  pub value : bool,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IoAnimation {
  IoAnimationInit,
  IoAnimationChangeState,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IoAnimationInit{
  pub name : String,
  pub repeat : i64,
  pub status : String,
  pub states : Vec<State>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct State {
  pub dulation: u64,
  pub state : PwmIOState, 
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PwmIOState {
  Io,
  Pwm,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct IoAnimationChangeState{
  pub name: String,
  pub status : String,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Ad{
  Stream(bool),
  None, // deinit
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Pwm {
  Io(u8),//init
  Freq(u64),
  Pulse(u64),
  Modulate(Modulate),
  None,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Modulate{ 
  pub r#type: String, 
  pub symbol_length: f64,
  pub data : Vec<u8>
}




#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Uart {
  UartInit,
  UartSend,
  None, // deinit
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UartInit {
  pub rx: u8,
  pub tx: u8,
  pub baud: u64,
  pub stop: u8,
  pub bits : u8,
  pub parity: u8,
  pub flowcontrol:String,
  pub rts: u8,
  pub cts: u8,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct UartSend {
  pub data : Vec<u8>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Spi {
  SpiInitMaster,
  None , // deinit
  SpiWrite,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SpiInitMaster {
  pub mode : String,
  pub clk: u8,
  pub mosi: u8,
  pub miso: u8,
  pub clock: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SpiWrite {
  pub data : Vec<u64>,
  pub read : bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum I2c {
  I2cInitMaster,
  I2cInitSlave,
  I2cWrite,
  I2cRead,
  None,// deinit
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct I2cInitMaster {
  pub mode : String,
  pub sda : u8,
  pub scl : u8,
  pub clock : u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct I2cInitSlave {
  pub mode: String,
  pub sda : u8,
  pub scl : u8,
  pub slave_address: u16,
  pub slave_address_length: u16,
  pub address: u16,
  pub address_bits: u16,
  pub data: Vec<u8>,
  pub read: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct I2cWrite {
  pub address: u16,
  pub address_bits: u16,
  pub data : Vec<u16>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct I2cRead {
  pub address: u16,
  pub address_bits: u16,
  pub read : u16,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicAnalyzer {
  LogicAnalyzerInit,
  None, // deinit
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LogicAnalyzerInit {
  pub io: Vec<u64>,
  pub interval: u64,
  pub dulation: u64,
  pub trigger: Trigger,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Trigger {
  pub value: bool,
  pub samples: u64,
}



#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Measure {
  pub echo: Echo,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Echo {
  pub io_pulse: i64,
  pub io_echo: i64,
  pub pulse: String,
  pub pulse_width: f64,
  pub mesure_edges: u8,
  pub timeout:f64,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Display {
  Text(String),
  Clear(bool),
  Qr(Qr),
  Raw,
  PinAssign(PinAssign),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Raw {
  raw:Vec<i64>,
  color_depth:i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PinAssign {
  #[serde(rename="0")]
  P0(Pin),
  #[serde(rename="1")]
  P1(Pin),
  #[serde(rename="2")]
  P2(Pin),
  #[serde(rename="3")]
  P3(Pin),
  #[serde(rename="4")]
  P4(Pin),
  #[serde(rename="5")]
  P5(Pin),
  #[serde(rename="6")]
  P6(Pin),
  #[serde(rename="7")]
  P7(Pin),
  #[serde(rename="8")]
  P8(Pin),
  #[serde(rename="9")]
  P9(Pin),
  #[serde(rename="10")]
  P10(Pin),
  #[serde(rename="11")]
  P11(Pin),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Pin {
  pub module_name: String,
  pub pin_name: String,
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Qr {
  text: String,
  correction: String,
}



#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Tcp {
  Connect{port : i64 , domain : String},
  Disconect(bool),
  Write{data:Vec<i64>},
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Wifi {
  scan : bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Ble {
  pub hci : Hci,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Hci {
    Initialize(bool),
    None,
    Write(Vec<i64>),
    AdvertisementFilter(Vec<AdvertisementFilterStruct>)
}

#[derive(Debug, Serialize)]
#[serde(rename="")]
pub struct AdvertisementFilterStruct{
  range : Range,
  value : Vec<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Range {
  index : i64,
  length : i64,
}

//example
// [
// {
//   "message": {
//       "data": "button pressed",
//       "to": [
//           "1234-5678"
//       ]
//   }
// }
// ]
// 
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Message {
  data : String,
  to: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Plugin {
  pub send : Vec<i64>,
}



#[cfg(test)]
mod api_tests {
  use super::*;
  #[test]
  fn test_ws_serialize() { 
    let obj = vec![Request::Ws(WS{reset_obniz_on_ws_disconnection:false,})];
    let serialized = serde_json::to_string(&obj).unwrap();
    let expected = r#"[{"ws":{"reset_obniz_on_ws_disconnection":false}}]"#;
    assert_eq!(serialized,expected.to_string(),)   
  }
  
  #[test]
  fn test_plugin_serialize() { 
    // let obj = vec![Request::WS(WS::ResetObnizOnWsDisconnection(false))];
    // let serialized = serde_json::to_string(&obj).unwrap();
    // let expected = r#"[{"ws":{"reset_obniz_on_ws_disconnection":false}}]"#;
    // assert_eq!(serialized,expected.to_string(),)   
  }


  // #[test]
  // fn test_ws_obniz_deserialize() {

  //   assert!(false);
  // }

}
