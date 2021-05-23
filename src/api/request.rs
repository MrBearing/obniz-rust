use serde::{Serialize};

#[derive(Debug,Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Request {
  Ws(WS),
  System(System),
}

#[derive(Debug,Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WS {
  ResetObnizOnWsDisconnection(bool),
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




#[cfg(test)]
mod api_tests {
  use super::*;
  #[test]
  fn test_ws_serialize() {
    let obj = vec![Request::Ws(WS::ResetObnizOnWsDisconnection(false))];
    let serialized = serde_json::to_string(&obj).unwrap();
    let expected = r#"[{"ws":{"reset_obniz_on_ws_disconnection":false}}]"#;
    assert_eq!(serialized,expected.to_string(),)   
  }


}
