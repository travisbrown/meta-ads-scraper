pub mod integer_str {
    use serde::{
        de::{Deserializer, Unexpected, Visitor},
        ser::Serializer,
    };
    use std::{marker::PhantomData, str::FromStr};

    const EXPECTED: &str = "integer string";

    pub fn deserialize<'de, T: FromStr, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<T, D::Error> {
        struct IntegerStrVisitor<T> {
            _target: PhantomData<T>,
        }

        impl<'de, T: FromStr> Visitor<'de> for IntegerStrVisitor<T> {
            type Value = T;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(EXPECTED)
            }

            fn visit_borrowed_str<E: serde::de::Error>(
                self,
                v: &'de str,
            ) -> Result<Self::Value, E> {
                v.parse::<Self::Value>()
                    .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(v), &EXPECTED))
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                v.parse::<Self::Value>()
                    .map_err(|_| serde::de::Error::invalid_value(Unexpected::Str(v), &EXPECTED))
            }
        }

        deserializer.deserialize_str(IntegerStrVisitor::<T> {
            _target: PhantomData,
        })
    }

    pub fn serialize<T: std::fmt::Display, S: Serializer>(
        value: &T,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }
}

pub mod integer_str_opt {
    use serde::{
        de::{Deserializer, Visitor},
        ser::Serializer,
    };
    use std::{marker::PhantomData, str::FromStr};

    const EXPECTED: &str = "optional integer string";

    pub fn deserialize<'de, T: FromStr, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<T>, D::Error> {
        struct IntegerStrOptVisitor<T> {
            _target: PhantomData<T>,
        }

        impl<'de, T: FromStr> Visitor<'de> for IntegerStrOptVisitor<T> {
            type Value = Option<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(EXPECTED)
            }

            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(None)
            }

            fn visit_some<D: Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, D::Error> {
                super::integer_str::deserialize(deserializer).map(Some)
            }
        }

        deserializer.deserialize_option(IntegerStrOptVisitor::<T> {
            _target: PhantomData,
        })
    }

    pub fn serialize<T: std::fmt::Display, S: Serializer>(
        value: &Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serializer.serialize_str(&value.to_string()),
            None => serializer.serialize_none(),
        }
    }
}

/// Deserialize an array of integer strings into a collection of integers (and the reverse).
pub mod integer_str_array {
    use serde::{
        de::{Deserializer, Visitor},
        ser::Serializer,
    };
    use std::iter::FromIterator;
    use std::marker::PhantomData;
    use std::str::FromStr;

    const EXPECTED: &str = "integer string array";

    pub fn deserialize<'de, E: FromStr, T: FromIterator<E>, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<T, D::Error> {
        struct IntegerStrArrayVisitor<E, T> {
            _element: PhantomData<E>,
            _target: PhantomData<T>,
        }

        impl<'de, E: FromStr, T: FromIterator<E>> Visitor<'de> for IntegerStrArrayVisitor<E, T> {
            type Value = T;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(EXPECTED)
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut error = std::cell::OnceCell::new();

                let wrapper: super::IntegerStrArraySeqAccessWrapper<'de, '_, A, E> =
                    super::IntegerStrArraySeqAccessWrapper {
                        underlying: seq,
                        error: &mut error,
                        _element: PhantomData,
                    };

                let result = T::from_iter(wrapper);

                error.take().map_or_else(|| Ok(result), |error| Err(error))
            }
        }

        deserializer.deserialize_seq(IntegerStrArrayVisitor::<E, T> {
            _element: PhantomData,
            _target: PhantomData,
        })
    }

    pub fn serialize<'a, E: std::fmt::Display, T: 'a, S: Serializer>(
        values: &'a T,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        &'a T: IntoIterator<Item = E>,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(None)?;

        for value in values {
            seq.serialize_element(&value.to_string())?;
        }

        seq.end()
    }
}

pub mod integer_str_array_opt {
    use serde::{
        de::{Deserializer, Visitor},
        ser::Serializer,
    };
    use std::iter::FromIterator;
    use std::marker::PhantomData;
    use std::str::FromStr;

    const EXPECTED: &str = "optional integer string array";

    pub fn deserialize<'de, E: FromStr, T: FromIterator<E>, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<T>, D::Error> {
        struct IntegerStrArrayOptVisitor<E, T> {
            _element: PhantomData<E>,
            _target: PhantomData<T>,
        }

        impl<'de, E: FromStr, T: FromIterator<E>> Visitor<'de> for IntegerStrArrayOptVisitor<E, T> {
            type Value = Option<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(EXPECTED)
            }

            fn visit_none<EE: serde::de::Error>(self) -> Result<Self::Value, EE> {
                Ok(None)
            }

            fn visit_some<D: Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> Result<Self::Value, D::Error> {
                super::integer_str_array::deserialize(deserializer).map(Some)
            }
        }

        deserializer.deserialize_option(IntegerStrArrayOptVisitor::<E, T> {
            _element: PhantomData,
            _target: PhantomData,
        })
    }

    pub fn serialize<'a, E: std::fmt::Display, T: 'a, S: Serializer>(
        values: &'a Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        &'a T: IntoIterator<Item = E>,
    {
        match values {
            Some(values) => super::integer_str_array::serialize(values, serializer),
            None => serializer.serialize_none(),
        }
    }
}

const INTEGER_STR_ARRAY_ELEMENT_EXPECTED: &str = "integer string";

struct IntegerStrArraySeqAccessWrapper<'de, 'a, A: serde::de::SeqAccess<'de>, E> {
    underlying: A,
    error: &'a mut std::cell::OnceCell<A::Error>,
    _element: std::marker::PhantomData<E>,
}

impl<'de, 'a, A: serde::de::SeqAccess<'de>, E: std::str::FromStr> IntoIterator
    for IntegerStrArraySeqAccessWrapper<'de, 'a, A, E>
{
    type Item = E;
    type IntoIter = IntegerStrArraySeqAccessIterator<'de, 'a, A, E>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter { source: self }
    }
}

struct IntegerStrArraySeqAccessIterator<'de, 'a, A: serde::de::SeqAccess<'de>, E> {
    source: IntegerStrArraySeqAccessWrapper<'de, 'a, A, E>,
}

impl<'de, A: serde::de::SeqAccess<'de>, E: std::str::FromStr> Iterator
    for IntegerStrArraySeqAccessIterator<'de, '_, A, E>
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.source.error.get().is_some() {
            None
        } else {
            match self
                .source
                .underlying
                .next_element::<std::borrow::Cow<'_, str>>()
            {
                Ok(Some(value)) => {
                    if let Ok(value) = value.parse() {
                        Some(value)
                    } else {
                        // We've just checked whether the cell is initialized.
                        self.source
                            .error
                            .set(serde::de::Error::invalid_value(
                                serde::de::Unexpected::Str(&value),
                                &INTEGER_STR_ARRAY_ELEMENT_EXPECTED,
                            ))
                            .unwrap();
                        None
                    }
                }
                Ok(None) => None,
                Err(error) => {
                    // We've just checked whether the cell is initialized.
                    self.source.error.set(error).unwrap();
                    None
                }
            }
        }
    }
}

pub mod timestamp_millis_str {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        String::deserialize(deserializer).and_then(|value| {
            let timestamp_ms = value
                .parse::<i64>()
                .map_err(|_| serde::de::Error::custom(format!("Invalid timestamp: {value}")))?;
            let timestamp = Utc
                .timestamp_millis_opt(timestamp_ms)
                .single()
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid timestamp: {value}")))?;

            Ok(timestamp)
        })
    }

    pub fn serialize<S: Serializer>(
        value: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.timestamp_millis().to_string())
    }
}

#[cfg(test)]
mod tests {
    #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    struct IntegerStrData {
        #[serde(with = "super::integer_str")]
        value: u64,
    }

    #[test]
    fn deserialize_integer_str() {
        let json = format!(r#"{{"value":"{}"}}"#, 123);
        let expected = IntegerStrData { value: 123 };

        assert_eq!(
            serde_json::from_str::<IntegerStrData>(&json).unwrap(),
            expected
        );
    }

    #[test]
    fn serialize_integer_str() {
        let value = IntegerStrData { value: 123 };
        let expected = format!(r#"{{"value":"{}"}}"#, 123);

        assert_eq!(serde_json::json!(value).to_string(), expected);
    }

    #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    struct IntegerStrOptData {
        #[serde(
            with = "super::integer_str_opt",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        value: Option<u64>,
    }

    #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    struct IntegerStrArrayData {
        #[serde(with = "super::integer_str_array")]
        values: Vec<u64>,
    }

    #[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
    struct IntegerStrArrayOptData {
        #[serde(
            with = "super::integer_str_array_opt",
            default,
            skip_serializing_if = "Option::is_none"
        )]
        values: Option<Vec<u64>>,
    }

    #[test]
    fn deserialize_some_integer_str_opt() {
        let json = format!(r#"{{"value":"{}"}}"#, 123);
        let expected = IntegerStrOptData { value: Some(123) };

        assert_eq!(
            serde_json::from_str::<IntegerStrOptData>(&json).unwrap(),
            expected
        );
    }

    #[test]
    fn serialize_some_integer_str_opt() {
        let value = IntegerStrOptData { value: Some(123) };
        let expected = format!(r#"{{"value":"{}"}}"#, 123);

        assert_eq!(serde_json::json!(value).to_string(), expected);
    }

    #[test]
    fn deserialize_missing_integer_str_opt() {
        let json = "{}";
        let expected = IntegerStrOptData { value: None };

        assert_eq!(
            serde_json::from_str::<IntegerStrOptData>(&json).unwrap(),
            expected
        );
    }

    #[test]
    fn deserialize_null_integer_str_opt() {
        let json = r#"{"value":null}"#;
        let expected = IntegerStrOptData { value: None };

        assert_eq!(
            serde_json::from_str::<IntegerStrOptData>(&json).unwrap(),
            expected
        );
    }
    #[test]
    fn serialize_none_integer_str_opt() {
        let value = IntegerStrOptData { value: None };
        let expected = "{}";

        assert_eq!(serde_json::json!(value).to_string(), expected);
    }

    #[test]
    fn deserialize_integer_str_array() {
        let json = r#"{"values":["123", "456"]}"#;
        let expected = IntegerStrArrayData {
            values: vec![123, 456],
        };

        assert_eq!(
            serde_json::from_str::<IntegerStrArrayData>(&json).unwrap(),
            expected
        );
    }

    #[test]
    fn serialize_integer_str_array() {
        let value = IntegerStrArrayData {
            values: vec![123, 456],
        };
        let expected = r#"{"values":["123","456"]}"#;

        assert_eq!(serde_json::json!(value).to_string(), expected);
    }

    #[test]
    fn deserialize_invalid_integer_str_array() {
        let invalid_type_json = r#"{"values":["123", 987, "456"]}"#;
        let invalid_value_json = r#"{"values":["123", "abc", "456"]}"#;

        let invalid_type_result = serde_json::from_str::<IntegerStrArrayData>(&invalid_type_json);
        let invalid_value_result = serde_json::from_str::<IntegerStrArrayData>(&invalid_value_json);

        assert!(invalid_type_result.is_err());
        assert!(invalid_value_result.is_err());
    }

    #[test]
    fn deserialize_integer_str_array_opt() {
        let json = r#"{"values":["123", "456"]}"#;
        let expected = IntegerStrArrayOptData {
            values: Some(vec![123, 456]),
        };

        assert_eq!(
            serde_json::from_str::<IntegerStrArrayOptData>(&json).unwrap(),
            expected
        );
    }

    #[test]
    fn serialize_integer_str_array_opt() {
        let value = IntegerStrArrayOptData {
            values: Some(vec![123, 456]),
        };
        let expected = r#"{"values":["123","456"]}"#;

        assert_eq!(serde_json::json!(value).to_string(), expected);
    }
}
