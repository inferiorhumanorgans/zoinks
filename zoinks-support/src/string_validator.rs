use std::fmt;

use quote::{ToTokens, quote};
use serde::de::{self, Unexpected, Visitor};

#[derive(Debug)]
pub struct StringValidatorConfig {
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
}

impl Default for StringValidatorConfig {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
        }
    }
}

impl ToTokens for StringValidatorConfig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let min_length = match self.min_length.as_ref() {
            Some(min_length) => quote!{Some(#min_length)},
            None => quote!{None}
        };

        let max_length = match self.max_length.as_ref() {
            Some(max_length) => quote!{Some(#max_length)},
            None => quote!{None}
        };

        tokens.extend(quote!{
            zoinks_support::StringValidatorConfig {
                min_length: #min_length,
                max_length: #max_length,
            }
        })
    }
}

impl StringValidatorConfig {
    fn valid(&self, s: &str) -> bool {
        let len = s.len();

        if let Some(min_length) = self.min_length {
            if (min_length as usize) > len {
                return false
            }
        }

        if let Some(max_length) = self.max_length {
            if (max_length as usize) < len {
                return false
            }
        }

        true
    }
}

impl<'de> Visitor<'de> for StringValidatorConfig {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let expected_string = format!("expected a string within length bounds {:?}, {:?}", self.min_length, self.max_length);
        formatter.write_str(&expected_string)
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        if self.valid(s) {
            Ok(s.into())
        } else {
            Err(de::Error::invalid_value(Unexpected::Str(s), &self))
        }
    }

}
