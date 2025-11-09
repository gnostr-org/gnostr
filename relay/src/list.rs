//! Deserialize a JSON string or array of strings into a Vec.
//! The strings separated by whitespace.

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Deref, DerefMut};
use std::{fmt, marker::PhantomData};

#[derive(Default, Clone, Debug)]
pub struct List(pub Vec<String>);
impl<'de> Deserialize<'de> for List {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        string_or_seq_string(deserializer).map(List)
    }
}

impl Serialize for List {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl Deref for List {
    type Target = Vec<String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for List {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<String>> for List {
    fn from(v: Vec<String>) -> Self {
        Self(v)
    }
}

fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value.split(' ').map(|s| s.to_owned()).collect())
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn list() -> anyhow::Result<()> {
        let li: List = serde_json::from_str("[\"a\", \"b\"]")?;
        assert_eq!(li[0], "a");
        assert_eq!(li[1], "b");
        let li: List = serde_json::from_str("\"a b\"")?;
        assert_eq!(li[0], "a");
        assert_eq!(li[1], "b");
        let li: List = serde_json::from_str("\"a\"")?;
        assert_eq!(li[0], "a");
        assert_eq!(li.len(), 1);
        Ok(())
    }

    #[test]
    fn test_empty_string() -> anyhow::Result<()> {
        //let json_str = r#"{ "name": "" }"#;
        let li: List = serde_json::from_str("[]")?;
        assert!(li.is_empty());
        Ok(())
    }

    #[test]
    fn test_empty_array() -> anyhow::Result<()> {
        let li: List = serde_json::from_str(r"[]")?;
        assert!(li.is_empty());
        Ok(())
    }

    #[test]
    fn test_string_with_multiple_spaces() -> anyhow::Result<()> {
        let li: List = serde_json::from_str("\"a b c\"")?;
        assert_eq!(li.len(), 3);
        assert_eq!(li[0], "a");
        assert_eq!(li[1], "b");
        assert_eq!(li[2], "c");
        Ok(())
    }

    #[test]
    fn test_string_with_leading_trailing_spaces() -> anyhow::Result<()> {
        let li: List = serde_json::from_str("\" a         b         \"")?;
        assert_eq!(li.len(), 20);
        assert_eq!(li[1], "a");
        assert_eq!(li[10], "b");
        Ok(())
    }

    #[test]
    fn test_serialization() -> anyhow::Result<()> {
        let li = List(vec!["c".to_string(), "d".to_string()]);
        let serialized = serde_json::to_string(&li)?;
        assert_eq!(serialized, "[\"c\",\"d\"]");

        let li: List = serde_json::from_str("\"e f\"")?;
        let serialized = serde_json::to_string(&li)?;
        assert_eq!(serialized, "[\"e\",\"f\"]");
        Ok(())
    }

    #[test]
    fn test_deref_deref_mut() {
        let mut li = List(vec!["g".to_string(), "h".to_string()]);
        assert_eq!(li.len(), 2);
        assert_eq!(li[0], "g");

        li.push("i".to_string());
        assert_eq!(li.len(), 3);
        assert_eq!(li[2], "i");

        li[0] = "j".to_string();
        assert_eq!(li[0], "j");
    }

    #[test]
    fn test_from_vec() {
        let vec = vec!["k".to_string(), "l".to_string()];
        let li = List::from(vec.clone());
        assert_eq!(li.0, vec);
    }
}
