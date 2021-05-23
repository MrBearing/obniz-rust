use serde::{Deserialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
  Ws(WS),
  System(System)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WS {
  Ready(bool),
  Obniz(Obniz),
  Redirect(String)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum System {
  Pon{key :Vec<i64> },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Obniz {
  pub hw: String,
  pub firmware: String,
  pub metadata: String
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



}
