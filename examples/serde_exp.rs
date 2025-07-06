use serde_json::Value;

fn main() {
    let data = r#"
  {
      "name": "John Doe",
      "age": 43,
      "phones": [
          "+44 1234567",
          "+44 2345678"
      ]
  }"#;

    // Parse the string of data into serde_json::Value.
    let v: Value = serde_json::from_str(data).expect("fail ..");

    // Do things just like with any other Rust data structure.
    println!("Please call {} at the number {}", v["name"], v["phones"][0]);
}
