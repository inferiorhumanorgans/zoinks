use std::fmt;

use quote::{ToTokens, quote};
use serde::de::{self, Unexpected, Visitor};

#[derive(Debug)]
pub struct NumericValidatorConfig {
    pub min: Option<f64>,
    pub exclusive_min: Option<f64>,
    pub max: Option<f64>,
    pub exclusive_max: Option<f64>,
}

impl Default for NumericValidatorConfig {
    fn default() -> Self {
        Self {
            min: None,
            exclusive_min: None,
            max: None,
            exclusive_max: None,
        }
    }
}

impl ToTokens for NumericValidatorConfig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let min = match self.min.as_ref() {
            Some(min) => quote!{Some(#min)},
            None => quote!{None}
        };

        let exclusive_min = match self.exclusive_min.as_ref() {
            Some(exclusive_min) => quote!{Some(#exclusive_min)},
            None => quote!{None}
        };

        let max = match self.max.as_ref() {
            Some(max) => quote!{Some(#max)},
            None => quote!{None}
        };

        let exclusive_max = match self.exclusive_max.as_ref() {
            Some(exclusive_max) => quote!{Some(#exclusive_max)},
            None => quote!{None}
        };

        tokens.extend(quote!{
            zoinks_support::NumericValidatorConfig {
                min: #min,
                exclusive_min: #exclusive_min,
                max: #max,
                exclusive_max: #exclusive_max,
            }
        })
    }
}

impl NumericValidatorConfig {
    fn valid(&self, n: f64) -> bool {
        if let Some(min) = self.min {
            if min > n {
                return false
            }
        }

        if let Some(exclusive_min) = self.exclusive_min {
            if exclusive_min >= n {
                return false
            }
        }

        if let Some(max) = self.max {
            if max < n {
                return false
            }
        }

        if let Some(exclusive_max) = self.exclusive_max {
            if exclusive_max <= n {
                return false
            }
        }

        true
    }
}

impl<'de> Visitor<'de> for NumericValidatorConfig {
    type Value = f64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let expected_string = format!("expected a number within bounds {:?}, {:?}", self.min, self.max);
        formatter.write_str(&expected_string)
    }

    fn visit_i64<E>(self, n: i64) -> Result<Self::Value, E>
        where E: de::Error
    {
        let float_value = n as f64;

        if self.valid(float_value) {
            Ok(float_value)
        } else {
            Err(de::Error::invalid_value(Unexpected::Signed(n), &self))
        }
    }
    fn visit_u64<E>(self, n: u64) -> Result<Self::Value, E>
        where E: de::Error
    {
        let float_value = n as f64;

        if self.valid(float_value) {
            Ok(float_value)
        } else {
            Err(de::Error::invalid_value(Unexpected::Unsigned(n), &self))
        }
    }

    fn visit_f64<E>(self, n: f64) -> Result<Self::Value, E>
        where E: de::Error
    {
        if self.valid(n) {
            Ok(n)
        } else {
            Err(de::Error::invalid_value(Unexpected::Float(n), &self))
        }
    }
}
