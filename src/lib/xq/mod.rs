//! Basic XQ-like query implementation for gnostr
//!
//! This provides simplified JQ-like query processing capabilities
//! for use within gnostr ecosystem.

//! Basic XQ-like query implementation for gnostr
//!
//! This provides simplified JQ-like query processing capabilities
//! for use within gnostr ecosystem.

use serde_json::{json, Value};
use std::io::{self, Read};

/// Error type for XQ operations
#[derive(Debug, thiserror::Error)]
pub enum XqError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Execution error: {0}")]
    Execution(String),
}

/// Result type for XQ operations
pub type XqResult<T> = Result<T, XqError>;

/// Simplified XQ processor
pub struct XqProcessor {
    json_value: Value,
}

impl XqProcessor {
    /// Create a new XQ processor from JSON input
    pub fn from_json(json_str: &str) -> XqResult<Self> {
        let json_value: Value = json_str
            .parse()
            .map_err(|e| XqError::Parse(e.to_string()))?;

        Ok(Self { json_value })
    }

    /// Execute a simple identity query (just returns the input)
    pub fn identity(&self) -> XqResult<Value> {
        // For now, just return the original value as-is
        Ok(self.json_value.clone())
    }

    /// Execute a basic field access query
    pub fn field_access(&self, field: &str) -> XqResult<Value> {
        match &self.json_value {
            Value::Object(obj) => {
                if let Some(value) = obj.get(field) {
                    Ok(value.clone())
                } else {
                    Err(XqError::Execution(format!("Field '{}' not found", field)))
                }
            }
            _ => Err(XqError::Execution(
                "Field access only works on objects".to_string(),
            )),
        }
    }

    /// Execute a basic array filter
    pub fn filter_objects(&self, predicate: impl Fn(&Value) -> bool) -> XqResult<Value> {
        match &self.json_value {
            Value::Array(arr) => {
                let filtered: Vec<Value> =
                    arr.iter().filter(|item| predicate(item)).cloned().collect();
                Ok(Value::Array(filtered))
            }
            _ => Err(XqError::Execution(
                "Filter only works on arrays".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_json_parsing() -> XqResult<()> {
        let processor = XqProcessor::from_json(r#"{"name": "test", "value": 123}"#)?;

        let result = processor.identity()?;
        assert!(matches!(result, Value::Object(_)));
        Ok(())
    }

    #[test]
    fn test_field_access() -> XqResult<()> {
        let processor = XqProcessor::from_json(r#"{"field": "value", "nested": {"key": "data"}}"#)?;

        let result = processor.field_access("field")?;
        assert!(matches!(result, Value::String(String::from("value"))));

        let result2 = processor.field_access("nested.key")?;
        assert!(matches!(result2, Value::String(String::from("data"))));
        Ok(())
    }

    #[test]
    fn test_array_filtering() -> XqResult<()> {
        let processor =
            XqProcessor::from_json(r#"[{"keep": true}, {"remove": false}, {"keep": false}]"#)?;

        let result = processor.filter_objects(|v| matches!(v, Value::Bool(true)))?;

        if let Value::Array(filtered) = result {
            assert_eq!(filtered.len(), 1);
        } else {
            panic!("Expected array result");
        }
        Ok(())
    }
}
