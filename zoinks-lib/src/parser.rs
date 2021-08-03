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
    #[serde(rename="$ref")]
    pub reference: Option<Cow<'a, str>>,

    #[serde(rename="$schema")]
    pub schema_uri: Option<Cow<'a, str>>,

    // https://github.com/serde-rs/serde/issues/1413
    // https://json-schema.org/draft/2020-12/json-schema-validation.html §9.1
    pub description: Option<Cow<'a, str>>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §9.1
    pub title: Option<Cow<'a, str>>,

    // alias: $defs
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
    pub multiple_of: Option<f64>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.2.2
    pub maximum: Option<f64>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.2.3
    pub exclusive_maximum: Option<f64>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.2.4
    pub minimum: Option<f64>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.2.5
    pub exclusive_minimum: Option<f64>,

    // 6.3. Validation Keywords for Strings
    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.3.1
    // Must be non-negative
    pub max_length: Option<u32>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.3.2
    // Must be non-negative
    pub min_length: Option<u32>,

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §6.3.3
    // Validate that it's a regex
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

    // https://json-schema.org/draft/2020-12/json-schema-validation.html §7
    pub format: Option<Cow<'a, str>>,
}
