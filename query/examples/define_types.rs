use serde::{Deserialize, Serialize};
use serde_json::Result;

// Define structs that mirror the JSON structure
#[derive(Debug, Deserialize, Serialize)]
struct Outer {
    level1_key: Inner,
    other_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Inner {
    level2_key_a: String,
    level2_key_b: i64,
    another_nested: DeepInner,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeepInner {
    deep_key: bool,
}

fn main() -> Result<()> {
    let json_data = r#"
        {
            "level1_key": {
                "level2_key_a": "value_a",
                "level2_key_b": 123,
                "another_nested": {
                    "deep_key": true
                }
            },
            "other_key": "some_other_value"
        }
    "#;

    // Deserialize the JSON string directly into your Rust struct
    let parsed_data: Outer = serde_json::from_str(json_data)?;

    // Access the second-level key directly through struct fields
    let level2_value_a = &parsed_data.level1_key.level2_key_a;
    let level2_value_b = parsed_data.level1_key.level2_key_b;
    let deep_value = parsed_data.level1_key.another_nested.deep_key;

    println!("Value of level2_key_a: {}", level2_value_a);
    println!("Value of level2_key_b: {}", level2_value_b);
    println!("Value of deep_key: {}", deep_value);

    // If a field is missing or has the wrong type, serde_json::from_str will return an Err
    // let invalid_json = r#"{ "level1_key": { "level2_key_a": "oops" } }"#;
    // let _ = serde_json::from_str::<Outer>(invalid_json); // This would return an Error

    Ok(())
}
