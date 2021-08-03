use proc_macro2::{TokenStream as TokenStream2};
use heck::{CamelCase, SnakeCase};
// use inflector::Inflector;
// use syn::Ident;
// use quote::{format_ident, quote};
use serde_json::Value as JsonValue;
use std::collections::HashSet;
use super::Schema;
use std::iter::FromIterator;
use regex::Regex;

#[allow(unused)]
use log::{info, error, warn, debug};

type OutVec = Vec<String>;

fn sanitize(before: &str) -> String {
    let out = before
        .replace("[]", "Array")
        .replace("<", "_Qlt_")
        .replace(">", "_Qgt_")
        .replace("|", "_Qor_")
        .replace(",", "_Qand_")
        .replace("(", "_Qop_")
        .replace(")", "_Qcp_")
        .replace("$", "_Reserved_")
        ;
    let re = Regex::new(r#"[\[\]"\-]"#).unwrap();
    // "ValueDef<(number|\"width\"|\"height\"|ExprRef)>": {
    let out = re.replace_all(&out, "_").to_string();
    match out.as_ref() {
        "as" => "reserved_as".into(),
        "type" => "reserved_type".into(),
        "box" => "reserved_box".into(),
        "URI" => "UniformResoruceIdiot".into(),
        s @ _ => s.into(),
    }
}

fn descend(in_name: String, schema: &Schema, out: &mut OutVec, root: bool) -> Option<String> {
    let name = sanitize(&in_name).to_camel_case();

    let instance_types : HashSet<String> = HashSet::from_iter(schema.instance_type.iter().cloned());

    // let strings_nulls : HashSet<String> = HashSet::from_iter(["string".into(), "null".into()]);

    if schema.any_of.len() > 0 && instance_types.is_empty() {
        let mut outer = vec![];
        outer.push(format!("/// any_of enum: {}", name));
        outer.push(format!("#[derive(Debug, serde::Deserialize)]"));
        outer.push(format!("#[serde(untagged)]"));
        outer.push(format!("pub enum {} {{", name));

        for (i, any) in schema.any_of.iter().enumerate() {
            match descend(format!("{}Variant{}", name, i), any, out, false) {
                Some(variant) => {
                    let variant = sanitize(&variant);
                    outer.push(format!("  {}({}),", variant, variant))
                },
                None => error!("Invalid: {}{}", name, i)
            }
        }
        outer.push(format!("}}"));
        out.extend(outer);
        return Some(name)
    } else if schema.enums.len() > 0 {
        let allow_string = instance_types.contains("string");
        let allow_number = instance_types.contains("number");
        let allow_null = instance_types.contains("null");

        let enums = schema.enums
            .iter()
            .filter_map(|e| {
                match e {
                    JsonValue::String(s) => {
                        if allow_string {
                            Some((s.into(), s.to_camel_case()))
                        } else {
                            warn!("Got a string enum value, but string values not allowed: {:?}", s.to_camel_case());
                            None
                        }
                    }
                    JsonValue::Number(n) => {
                        if allow_number {
                            Some((format!("{}", n), format!("{}_{}", name, n).to_camel_case()))
                        } else {
                            warn!("Got a numeric enum value, but number values not allowed: {:?}", n);
                            None
                        }
                    },
                    JsonValue::Null => {
                        if allow_null {
                            Some((format!("null"), format!("Null")))
                        } else {
                            warn!("Got a null enum value, but null values not allowed");
                            None
                        }
                    },
                    _ => todo!("unsupported enum type: {:?}", e)
                }
            })
            .map(|(old_s, new_s)| {
                if *old_s != new_s {
                    format!(r#"#[serde(rename="{}")] {}"#, old_s, new_s)
                } else {
                    format!("{}", new_s)
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        // out.push(format!("#[allow(non_camel_case_types)]"));
        out.push(format!("#[derive(Debug, serde::Deserialize)]"));
        out.push(format!("pub enum {} {{ {} }}", name, enums));

        return Some(name)
    } else if instance_types.len() == 1 && instance_types.contains("object") {
        if schema.properties.is_empty() {
            error!("{} Empty object specified", name);
            out.push(format!("#[derive(Debug, serde::Deserialize)]"));
            out.push(format!("pub struct {} (serde_json::Value);", name));

            return Some(name)
        } else {
            let mut outer = vec![];
            outer.push(format!("#[derive(Debug, serde::Deserialize)]"));
            outer.push(format!("pub struct {} {{", name));

            for (prop_name, prop_schema) in schema.properties.iter() {
                let prop_type = descend(format!("{}_prpty_{}", name, prop_name).to_camel_case(), &prop_schema, out, false).unwrap();

                let field_name = sanitize(prop_name).to_snake_case();
                if *prop_name != field_name {
                    outer.push(format!(r#"  #[serde(rename="{}")]"#, prop_name));
                }
                outer.push(format!("  pub {}: Option<{}>,", field_name, prop_type));
            }

            outer.push(format!("}}"));

            out.extend(outer);
            return Some(name)
        }
    } else if schema.reference.is_some() && instance_types.is_empty() {
        let reference = schema.reference.as_ref().unwrap();

        if reference.starts_with("#/definitions/") {
            let reference = &reference[14..];
            let reference = sanitize(reference).to_camel_case();
            if root {
                out.push(format!("#[derive(Debug, serde::Deserialize)]"));
                out.push(format!("pub struct {}({});", name, reference))
            }

            return Some(reference.into())
        } else {
            error!("Unsupported reference: {}", reference);
            return None
        }
    } else if instance_types.len() == 1 && instance_types.contains("number") {
        out.push(format!("#[derive(Debug, serde::Deserialize)]"));
        out.push(format!("pub struct {}(f64);", name));

        return Some(name)
    } else if instance_types.len() == 1 && instance_types.contains("boolean") {
        out.push(format!("#[derive(Debug, serde::Deserialize)]"));
        out.push(format!("pub struct {}(bool);", name));

        return Some(name)
    } else if instance_types.len() == 1 && instance_types.contains("string") {
        out.push(format!("#[derive(Debug, serde::Deserialize)]"));
        out.push(format!("pub struct {}(String);", name));

        return Some(name)
    } else if instance_types.len() == 1 && instance_types.contains("null") {
        return Some("Null".into())
    } else if instance_types.len() == 1 && instance_types.contains("array") {
        let inner_name = format!("VecOf{}", name.to_camel_case());
        let inner_name = match descend(inner_name.clone(), schema.items.as_ref().unwrap(), out, false) {
            Some(name) => name,
            None => inner_name,
        };

        // out.push(format!("#[derive(Debug, serde::Deserialize)]"));
        out.push(format!("pub type {} = Vec<{}>;", name, sanitize(&inner_name)));
        return Some(name)
    } else if instance_types.len() > 1 {
        let enums = instance_types.iter()
            .map(|e| match e.as_ref() {
                "number" => "Number(f64)",
                "string" => "String(String)",
                "boolean" => "Boolean(bool)",
                "null" => "Null",
                _ => unimplemented!(),
            })
            .collect::<Vec<_>>()
            .join(", ");

        out.push(format!("#[derive(Debug, serde::Deserialize)]"));
        out.push(format!("pub enum {} {{ {} }}", name, enums));
        return Some(name)
    } else {
        error!("Unsupported {}: {:?}", name, schema);
        out.push(format!("#[derive(Debug, serde::Deserialize)]"));
        out.push(format!("pub struct {};", name));

        return Some(name)
    }
}

pub fn genimpl(schema: &Schema) -> TokenStream2 {
    let mut out : OutVec = vec![];

    for (name, defn) in schema.definitions.iter() {
        descend(name.into(), &defn, &mut out, true);
    }

    // println!("#![allow(non_camel_case_types)]");
    println!("#[derive(Debug, serde::Deserialize)]");
    println!("pub struct Null;");

    for foo in out.iter() {
        println!("{}", foo)
    }

    todo!()
}
