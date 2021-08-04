use std::borrow::Cow;
use std::collections::HashSet;
use std::iter::FromIterator;

use heck::{CamelCase, SnakeCase};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use regex::Regex;
use serde_json::Value as JsonValue;

#[allow(unused)]
use log::{info, error, warn, debug};

use super::Schema;

mod tokens;
use tokens::*;

type OutVec = Vec<RustItem>;

// TODO: Make this more generic
fn is_boxed(type_name: &str, field_name: &str, prop_count: usize) -> bool {
    // If we have a mega struct let's box the fields
    if prop_count > 15 {
        return true
    }

    //  Because of recursion
    if type_name == "DerivedStream" && field_name == "stream" {
        return true
    }

    if type_name == "LogicalNotQltPredicateQgt" && field_name == "not" {
        return true
    }

    if type_name == "NonLayerRepeatSpec" && field_name == "spec" {
        return true
    }

    return false
}

fn sanitize(before: &str) -> String {
    let out = before
        .replace("[]", "_Array_")
        .replace("<", "_Qlt_")
        .replace(">", "_Qgt_")
        .replace("|", "_Qor_")
        .replace(",", "_Qand_")
        .replace("(", "_Qop_")
        .replace(")", "_Qcp_")
        .replace("$", "_Reserved_")
        ;
    let re = Regex::new(r#"[\[\]"\-]"#).unwrap();
    match re.replace_all(&out, "_") {
        Cow::Borrowed("as") => "reserved_as".into(),
        Cow::Borrowed("type") => "reserved_type".into(),
        Cow::Borrowed("box") => "reserved_box".into(),
        Cow::Borrowed("URI") => "UniformResourceIdiot".into(),
        s @ _ => s.into(),
    }
}

fn descend(in_name: Cow<str>, schema: &Schema, out: &mut OutVec, root: bool) -> Option<String> {
    let name = sanitize(&in_name).to_camel_case();

    let instance_types : HashSet<String> = HashSet::from_iter(schema.instance_type.iter().cloned());

    // let strings_nulls : HashSet<String> = HashSet::from_iter(["string".into(), "null".into()]);

    if schema.any_of.len() > 0 && instance_types.is_empty() {
        let mut outer = Vec::with_capacity(4);
        outer.push(RustItem::DocComment(format!("any_of enum: {}", name)));
        outer.push(RustItem::DeriveCommon);
        outer.push(RustItem::SerdeUntagged);

        let mut variants = Vec::with_capacity(schema.any_of.len());

        for (i, any) in schema.any_of.iter().enumerate() {
            match descend(format!("{}Variant{}", name, i), any, out, false) {
                Some(variant) => {
                    let variant = sanitize(&variant);
                    variants.push(EnumVariant::Tuple(variant.clone(), variant.clone()));
                },
                None => error!("Invalid: {}{}", name, i)
            }
        }
        outer.push(RustItem::Enum(Enum {
            name: name.clone(),
            variants,
        }));
        out.extend(outer);
        return Some(name)
    } else if schema.enums.len() > 0 {
        let allow_string = instance_types.contains("string");
        let allow_number = instance_types.contains("number");
        let allow_null = instance_types.contains("null");

        let variants = schema.enums
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
                            Some((String::from("null"), String::from("Null")))
                        } else {
                            warn!("Got a null enum value, but null values not allowed");
                            None
                        }
                    },
                    _ => todo!("unsupported enum type: {:?}", e)
                }
            })
            .map(|(old_name, new_name)| EnumVariant::Unit(new_name, old_name))
            .collect::<Vec<_>>();

        out.push(RustItem::DeriveCommon);
        out.push(RustItem::Enum(Enum {
            name: name.clone(),
            variants,
        }));

        return Some(name)
    } else if instance_types.len() == 1 && instance_types.contains("object") {
        if schema.properties.is_empty() {
            out.push(RustItem::DeriveCommon);
            out.push(RustItem::TupleStruct(name.clone(), "serde_json::Value".into()));

            return Some(name)
        } else {
            let mut outer = Vec::with_capacity(4);
            outer.push(RustItem::DeriveCommon);

            let prop_count = schema.properties.len();
            let mut fields = Vec::with_capacity(prop_count);

            for (prop_name, prop_schema) in schema.properties.iter() {
                let prop_type = descend(Cow::Owned(format!("{}_prpty_{}", name, prop_name).to_camel_case()), &prop_schema, out, false).unwrap();

                let field_name = sanitize(prop_name).to_snake_case();

                fields.push(StructField {
                    old_name: prop_name.to_string(),
                    field_type: prop_type,
                    required: false,
                    boxed: is_boxed(&name, &field_name, prop_count),
                    name: field_name,
                });
            }

            let fields = fields.into_iter()
                .map(|field| {
                    if schema.required.contains(&field.old_name) {
                        StructField {
                            required: true,
                            ..field
                        }
                    } else {
                        field
                    }
                })
                .collect();

            let additional_fields = match schema.additional_properties.as_ref() {
                None => true,
                Some(additional_properties) => match additional_properties {
                    crate::parser::AdditionalProperties::Boolean(b) if *b == false => false,
                    _ => true
                }
            };

            outer.push(RustItem::Struct(Struct {
                name: name.clone(),
                fields,
                additional_fields,
            }));

            out.extend(outer);
            return Some(name)
        }
    } else if schema.reference.is_some() && instance_types.is_empty() {
        let reference = schema.reference.as_ref().unwrap();

        if reference.starts_with("#/definitions/") {
            let reference = &reference[14..];
            let reference = sanitize(reference).to_camel_case();
            if root {
                out.push(RustItem::DeriveCommon);
                out.push(RustItem::TupleStruct(name, reference.clone()));
                // out.push(format!("pub struct {}({});", name, reference))
            }

            return Some(reference)
        } else {
            error!("Unsupported reference: {}", reference);
            return None
        }
    } else if instance_types.len() == 1 {
        if instance_types.contains("number") {
            out.push(RustItem::DeriveCommon);
            out.push(RustItem::TupleStruct(name.clone(), String::from("f64")));

            return Some(name)
        } else if instance_types.contains("boolean") {
            out.push(RustItem::DeriveCommon);
            out.push(RustItem::TupleStruct(name.clone(), String::from("bool")));

            return Some(name)
        } else if instance_types.contains("string") {
            out.push(RustItem::DeriveCommon);
            out.push(RustItem::TupleStruct(name.clone(), String::from("String")));

            return Some(name)
        } else if instance_types.contains("null") {
            return Some(String::from("Null"))
        } else if instance_types.contains("array") {
            let inner_name = format!("{}Item", name.to_camel_case());
            let inner_name = match descend(Cow::Borrowed(&inner_name), schema.items.as_ref().unwrap(), out, false) {
                Some(name) => name,
                None => inner_name,
            };

            out.push(RustItem::TypeAlias(name.clone(), format!("Vec<{}>", sanitize(&inner_name))));

            return Some(name)
        } else {
            todo!()
        }
    } else if instance_types.len() > 1 {
        let variants = instance_types.iter()
            .map(|e| match e.as_ref() {
                "number" => EnumVariant::Tuple(String::from("Number"), String::from("f64")),
                "string" => EnumVariant::Tuple(String::from("String"), String::from("String")),
                "boolean" => EnumVariant::Tuple(String::from("Boolean"), String::from("bool")),
                "null" => EnumVariant::Unit(String::from("Null"), String::from("null")),
                _ => unimplemented!(),
            })
            .collect::<Vec<_>>();

        out.push(RustItem::DeriveCommon);
        out.push(RustItem::Enum(Enum {
            name: name.clone(), variants
        }));
        // out.push(format!("pub enum {} {{ {} }}", name, enums));
        return Some(name)
    } else {
        warn!("Empty struct? {}", name);
        out.push(RustItem::DeriveCommon);
        out.push(RustItem::UnitStruct(name.clone()));

        return Some(name)
    }
}

pub fn genimpl(schema: &Schema) -> TokenStream2 {
    let mut out : OutVec = vec![
        RustItem::DeriveCommon,
        RustItem::UnitStruct(String::from("Null")),
    ];

    for (name, defn) in schema.definitions.iter() {
        descend(Cow::Borrowed(name.as_str()), &defn, &mut out, true);
    }

    quote! {
        #(#out)*
    }
}
