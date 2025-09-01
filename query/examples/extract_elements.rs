use serde_json::{Result, Value};
//use serde::de::Error;
use serde::ser::Error;

fn extract_elements(json_str: &str, keys: &[&str]) -> Result<Value> {
    let json: Value = serde_json::from_str(json_str)?;

    match json {
        Value::Object(map) => {
            let mut extracted = serde_json::Map::new();
            for key in keys {
                if let Some(value) = map.get(*key) {
                    extracted.insert(key.to_string(), value.clone());
                }
            }
            Ok(Value::Object(extracted))
        }
        _ => Err(serde_json::Error::custom("Input is not a JSON object")),
    }
}

fn main() {
    let json_str = r#"
        {
            "name": "John Doe",
            "age": 30,
            "city": "New York",
            "is_active": true,
            "address": {
                "street": "123 Main St",
                "zip": "10001"
            }
        }
    "#;

    let keys_to_extract = [
        //		"name",
        //		"age",
        //		"city",
        //		"is_active",
        "address", //		"street",
        "zip",
    ];

    match extract_elements(json_str, &keys_to_extract) {
        Ok(extracted_json) => {
            println!("1:Extracted JSON: {}", extracted_json);

            match extract_elements(&extracted_json.to_string(), &["address", "zip"]) {
                Ok(extracted_json) => {
                    println!("2:Extracted JSON: {}", extracted_json);
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }

    let json_str_array = r#"[
        {"name": "John Doe", "age": 30},
        {"name": "Jane Smith", "age": 25}
    ]"#;

    let keys_array = ["name"];

    let result_array: Result<Vec<Value>> =
        serde_json::from_str(json_str_array).map(|array: Vec<Value>| {
            array
                .into_iter()
                .map(|value| match value {
                    Value::Object(map) => {
                        let mut extracted = serde_json::Map::new();
                        for key in keys_array {
                            if let Some(val) = map.get(key) {
                                extracted.insert(key.to_string(), val.clone());
                            }
                        }
                        Value::Object(extracted)
                    }
                    _ => Value::Null,
                })
                .collect()
        });

    match result_array {
        Ok(extracted_array) => println!(
            "Extracted Array: {}",
            serde_json::to_string_pretty(&extracted_array).unwrap()
        ),
        Err(e) => eprintln!("Error extracting from array: {}", e),
    }

    let _ = levels();
}

fn levels() -> Result<()> {
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

    // Parse the JSON string into a serde_json::Value
    let parsed_json: Value = serde_json::from_str(json_data)?;

    // Access the second-level key using square brackets
    // This returns a `&Value` which can then be converted to the desired type
    let level2_value_a = &parsed_json["level1_key"]["level2_key_a"];
    let level2_value_b = &parsed_json["level1_key"]["level2_key_b"];

    // You can also chain `.get()` calls, which returns an Option<&Value>
    // This is safer if keys might be missing, as it avoids panicking
    let deep_value = parsed_json
        .get("level1_key")
        .and_then(|l1| l1.get("another_nested"))
        .and_then(|l2| l2.get("deep_key"));

    println!("Value of level2_key_a: {:?}", level2_value_a);
    println!("Value of level2_key_b: {:?}", level2_value_b);

    // To get the actual data, you often need to convert it from `&Value`
    if let Some(value_str) = level2_value_a.as_str() {
        println!("level2_key_a as string: {}", value_str);
    }

    if let Some(value_int) = level2_value_b.as_i64() {
        println!("level2_key_b as i64: {}", value_int);
    }

    if let Some(value_bool) = deep_value.and_then(|v| v.as_bool()) {
        println!("deep_key as bool: {}", value_bool);
    }

    // Handling missing keys:
    let non_existent_key = &parsed_json["level1_key"]["non_existent"];
    println!("Non-existent key: {:?}", non_existent_key); // Prints Null

    // Using .get() for safer access
    let non_existent_key_safe = parsed_json
        .get("level1_key")
        .and_then(|l1| l1.get("non_existent_safe"));
    println!("Non-existent key (safe): {:?}", non_existent_key_safe); // Prints None

    Ok(())
}
