use std::fmt;

use serde::{Deserialize, Deserializer};
use serde::de::{self, SeqAccess, Visitor};

mod numeric_validator;
pub use numeric_validator::NumericValidatorConfig;

mod string_validator;
pub use string_validator::StringValidatorConfig;

// https://github.com/serde-rs/serde/issues/889
pub fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where D: Deserializer<'de>
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(vec![s.to_owned()])
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
            where S: SeqAccess<'de>
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}
