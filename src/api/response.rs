use serde::{Deserialize};

// this is root node
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
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

  // IO Animation
  Io{animation : IoAnimation},
  // AD
  Ad0(f64),
  Ad1(f64),
  Ad2(f64),
  Ad3(f64),
  Ad4(f64),
  Ad5(f64),
  Ad6(f64),
  Ad7(f64),
  Ad8(f64),
  Ad9(f64),
  Ad10(f64),
  Ad11(f64),

  //uart 
  Uart0(Uart),
  Uart1(Uart),
  Spi0(Spi),
  Spi1(Spi),
  I2c0(I2c),
  LogicAnalyzer(LogicAnalyzer),
  Measure(Measure),
  Switch{state: String, action : String},
  Tcp0(Tcp),
  Wifi(Wifi),
  Ble(Ble),
  Message(Message),
  Debug(Debug),
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WS {
  Ready(bool),
  Obniz(Obniz),
  Redirect(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Obniz {
  pub hw: String,
  pub firmware: String,
  pub metadata: String
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum System {
  Pon{key :Vec<i64> },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Io {
  bool,
  Warning{message: String},
  Error{message: String},
}




#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct IoAnimation {
  pub name: String,
  pub status : String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Uart {
  data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Spi{
  data :Vec<i64>,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum I2c{
  I2cMaster,
  I2cSlave,
  Error{message: String},
  Warning{message: String},
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct I2cMaster{
  pub mode : String,
  pub address : i64,
  pub date : Vec<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct I2cSlave{
  pub mode : String,
  pub address : i64,
  pub is_flagmented: bool,
  pub date : Vec<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LogicAnalyzer {
  pub data : Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Measure {
  pub echo : Vec<Echo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Echo {
  pub edge: bool,
  pub timing: u32,
}

// TODO tcp0以外のtcpが存在しないか問い合わせ
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tcp {
  Read(Vec<i64>),
  Connect{message : String , code : i64},
  Connected(bool)
}

// pub enum ConnectMessage {
//   OK("ok");
// }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Wifi {
  scan: Vec<i64>,
}



#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Ble {
  hci : Hci,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Hci {
  read : Read,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Read {
  data : Vec<i64>,
}

///example
// [
//     {
//         "message": {
//             "data": "button pressed",
//             "from": "1234-5678"
//         }
//     }
// ]
/// 
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Message {
  pub data : String,
  pub from : String,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Plugin {
  pub receive : Vec<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Debug {
  Warning{message :String },
  Error{message: String},
}




#[cfg(test)]
mod api_tests {
  use super::*;
  #[test]
  fn test_ws_obniz_deserialize() {
    let text = r#"[{
      "ws": {
          "obniz": {
              "hw": "obnizb1",
              "firmware": "2.0.0",
              "metadata": "{\"description\":\"At My Office\"}"
          }
      }
    }]"#;
    let obj = serde_json::from_str::<Vec<Response>>(text);
    //assert!(obj.is_ok());
    let obj = obj.unwrap();
    if let Response::Ws(ws) = &obj[0] {
      if let WS::Obniz(obniz) = ws {
        assert_eq!(obniz.hw ,"obnizb1");
        assert_eq!(obniz.firmware , "2.0.0");
        assert_eq!(obniz.metadata , "{\"description\":\"At My Office\"}");
      }
    }
  }

  #[test]
  fn test_ws_redy_deserialize() {
    let text = r#"[
      {
          "ws": {
              "ready": true
          }
      }
    ]"#;
    let obj = serde_json::from_str::<Vec<Response>>(text);
    //assert!(obj.is_ok());
    let obj = obj.unwrap();
    if let Response::Ws(ws) = &obj[0] {
      if let WS::Ready(ready) = ws {
        assert!(*ready == true);
      }
    }
  }
  #[test]
  fn test_debug_deserialize() {
    let text = r#"[
      {
          "debug": {
              "warning": {
                  "message": "unknown command"
              }
          }
      }
    ]"#;
    let obj = serde_json::from_str::<Vec<Response>>(text);
    println!("{:?}", obj);
    let obj = obj.unwrap();
    if let Response::Debug(de) = &obj[0] {
      if let Debug::Warning{message} = de {
        assert_eq!(message, "unknown command");
      }
    }

    let text = r#"[{
          "debug": {
              "error": {
                  "message": "voltage down"
              }
          }
      }]"#;
    let obj = serde_json::from_str::<Vec<Response>>(text);
    println!("{:?}", obj);
    let obj = obj.unwrap();
    if let Response::Debug(de) = &obj[0] {
      if let Debug::Error{message} = de {
        assert_eq!(message, "voltage down");
      }
    }
  }
}
