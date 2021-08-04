use std::borrow::Cow;

use serde_json::Value as JsonValue;
use serde::Deserialize;
use indexmap::IndexMap;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum AdditionalProperties<'a> {
    Boolean(bool),
    #[serde(borrow)]
    Schema(Box<Schema<'a>>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase")]
#[serde(deny_unknown_fields)]
pub struct Schema<'a> {
    // https://json-schema.org/draft/2020-12/json-schema-core.html#rfc.section.8.2.1 §8.2.1
    #[serde(rename="$id")]
    pub id: Option<Cow<'a, str>>,

    #[serde(rename="$ref")]
    pub reference: Option<Cow<'a, str>>,

    #[serde(rename="$schema")]
    pub schema_uri: Option<Cow<'a, str>>,

    // https://github.com/serde-rs/serde/issues/1413
    // https://json-schema.org/draft/2020-12/json-schema-validation.html §9.1
    pub description: Option<Cow<'a, str>>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §9.1
    pub title: Option<Cow<'a, str>>,

    #[serde(alias = "$defs")]
    #[serde(default)]
    pub definitions: IndexMap<String, Schema<'a>>,

    #[serde(borrow)]
    pub additional_properties: Option<AdditionalProperties<'a>>,

    #[serde(default)]
    pub properties: IndexMap<Cow<'a, str>, Schema<'a>>,

    #[serde(default)]
    pub all_of: Vec<Schema<'a>>,

    #[serde(default)]
    pub any_of: Vec<Schema<'a>>,

    #[serde(default)]
    pub one_of: Vec<Schema<'a>>,

    #[serde(default)]
    pub items: Option<Box<Schema<'a>>>,

    #[serde(default)]
    #[serde(rename="enum")]
    pub enums: Vec<JsonValue>,

    #[serde(rename="type")]
    #[serde(default)]
    #[serde(deserialize_with = "zoinks_support::string_or_vec")]
    pub instance_type: Vec<String>,

    #[serde(default)]
    #[serde(deserialize_with = "zoinks_support::string_or_vec")]
    pub required: Vec<String>,

    // 6.2. Validation Keywords for Numeric Instances (number and integer)
    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.2.1
    /// The value of "multipleOf" MUST be a number, strictly greater than 0.
    /// A numeric instance is valid only if division by this keyword's value results in an integer.
    pub multiple_of: Option<f64>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.2.2
    /// The value of "maximum" MUST be a number, representing an inclusive upper limit for a numeric instance.
    /// If the instance is a number, then this keyword validates only if the instance is less than or exactly equal to "maximum".
    pub maximum: Option<f64>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.2.3
    pub exclusive_maximum: Option<f64>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.2.4
    /// The value of "minimum" MUST be a number, representing an inclusive lower limit for a numeric instance.
    /// If the instance is a number, then this keyword validates only if the instance is greater than or exactly equal to "minimum".
    pub minimum: Option<f64>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.2.5
    /// The value of "exclusiveMinimum" MUST be a number, representing an exclusive lower limit for a numeric instance.
    /// If the instance is a number, then the instance is valid only if it has a value strictly greater than (not equal to) "exclusiveMinimum".
    pub exclusive_minimum: Option<f64>,

    // 6.3. Validation Keywords for Strings
    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.3.1
    /// The value of this keyword MUST be a non-negative integer.
    /// A string instance is valid against this keyword if its length is less than, or equal to, the value of this keyword.
    pub max_length: Option<u32>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.3.2
    // Must be non-negative
    /// The value of this keyword MUST be a non-negative integer.
    /// A string instance is valid against this keyword if its length is greater than, or equal to, the value of this keyword.
    /// Omitting this keyword has the same behavior as a value of 0.
    pub min_length: Option<u32>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.3.3
    // TODO: Validate that it's a regex
    /// The value of this keyword MUST be a string. This string SHOULD be a valid regular expression, according to the ECMA-262 regular expression dialect.
    /// A string instance is considered valid if the regular expression matches the instance successfully. Recall: regular expressions are not implicitly anchored.
    pub pattern: Option<Cow<'a, str>>,

    // 6.4. Validation Keywords for Arrays
    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.4.1
    pub max_items: Option<u32>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.4.2
    pub min_items: Option<u32>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.4.3
    pub unique_items: Option<bool>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.1.3
    #[serde(rename="const")]
    pub constant: Option<JsonValue>,
    
    // 6.5. Validation Keywords for Objects
    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.5.1
    pub max_properties: Option<u32>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.5.2
    pub min_properties: Option<u32>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.5.4
    #[serde(default)]
    pub dependent_required: IndexMap<String, Vec<String>>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §7
    pub format: Option<Cow<'a, str>>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §9.2
    pub default: Option<JsonValue>,

    // https://json-schema.org/draft/2020-12/json-schema-core.html §10.2.1.4
    pub not: Option<Box<Schema<'a>>>,

    // https://json-schema.org/draft/2020-12/json-schema-core.html §10.3.2.2
    #[serde(default)]
    pub pattern_properties: IndexMap<String, Schema<'a>>,
}
