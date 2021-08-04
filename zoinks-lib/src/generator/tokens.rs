use proc_macro2::TokenStream as TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, quote, format_ident};
use zoinks_support::{StringValidatorConfig, NumericValidatorConfig};

#[derive(Debug)]
pub(super) enum EnumVariant {
    // {}({}),
    // name, type
    Tuple(String, String),

    // {},
    // name, old_name
    Unit(String, String),
}

#[derive(Debug)]
pub(super) struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug)]
pub(super) struct StructField {
    pub name: String,
    pub old_name: String,
    pub field_type: String,
    pub required: bool,
    pub boxed: bool,
    pub description: Option<String>,
}

#[derive(Debug)]
pub(super) struct Struct {
    pub name: String,
    pub fields: Vec<StructField>,
    pub additional_fields: bool,
}

#[derive(Debug)]
pub(super) enum RustItem {
    DocComment(String),

    // #[derive(Debug, serde::Deserialize)]
    DeriveCommon,

    // #[derive(Debug)]
    DeriveNoSerde,

    // #[serde(untagged)]
    SerdeUntagged,

    // e.g. pub type {} = Vec<{}>;
    // Name, type
    TypeAlias(String, String),

    Enum(Enum),

    // pub struct {} (serde_json::Value);
    // name, type
    TupleStruct(String, String),

    // pub struct {};", name));
    UnitStruct(String),

    Struct(Struct),

    StringValidator(String, StringValidatorConfig),
    NumericValidator(String, NumericValidatorConfig),
}

impl std::fmt::Display for StructField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let is_collection = self.field_type.starts_with("Vec");

        let field_type : std::borrow::Cow<str> = match self.boxed {
            true => format!("Box<{}>", self.field_type).into(),
            false => (&self.field_type).into(),
        };

        if self.name != self.old_name {
            writeln!(f, r#"  #[serde(rename="{}")]"#, self.old_name)?;
        }

        if self.required {
            writeln!(f, "  pub {}: {},", self.name, field_type)
        } else {
            if is_collection {
                writeln!(f, r#"  #[serde(default)]"#)?;
                writeln!(f, "  pub {}: {},", self.name, field_type)
            } else {
                writeln!(f, "  pub {}: Option<{}>,", self.name, field_type)
            }
        }
    }
}

impl std::fmt::Display for Struct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.additional_fields == false {
            writeln!(f, r#"#[serde(deny_unknown_fields)]"#)?;
        }

        writeln!(f, "pub struct {} {{", self.name)?;
        for field in self.fields.iter() {
            write!(f, "{}", field)?;
        }
        writeln!(f, "}}")
    }
}

impl std::fmt::Display for Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "pub enum {} {{", self.name)?;
        for variant in self.variants.iter() {
            write!(f, "{}", variant)?;
        }
        writeln!(f, "}}")
    }
}

impl std::fmt::Display for EnumVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit(variant_name, old_name) => {
                if variant_name != old_name {
                    writeln!(f, r#"  #[serde(rename="{}")]"#, old_name)?;
                }
                writeln!(f, "  {},", variant_name)
            },
            Self::Tuple(variant_name, type_name) => writeln!(f, "  {}({}),", variant_name, type_name),
        }
    }
}

impl std::fmt::Display for RustItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DocComment(text) => writeln!(f, "/// {}", text),
            Self::DeriveCommon => write!(f, "#[derive(Debug, serde::Deserialize)]"),
            Self::DeriveNoSerde => write!(f, "#[derive(Debug)]"),
            Self::SerdeUntagged => write!(f, "#[serde(untagged)]"),
            Self::TypeAlias(type_alias, type_source) => write!(f, "pub type {} = {};", type_alias, type_source),
            Self::Enum(enum_decl) => write!(f, "{}", enum_decl),
            Self::TupleStruct(struct_name, struct_type) => write!(f, "pub struct {} ({});", struct_name, struct_type),
            Self::UnitStruct(struct_name) => write!(f, "pub struct {};", struct_name),
            Self::Struct(struct_decl) => write!(f, "{}", struct_decl),
            Self::StringValidator(field_name, config) => write!(f, "String {} {:?}", field_name, config),
            Self::NumericValidator(field_name, config) => write!(f, "Numeric {} {:?}", field_name, config),
        }
    }
}

impl ToTokens for RustItem {
    fn to_tokens(&self, out: &mut TokenStream) {
        match self {
            Self::DocComment(text) => {
                let doc_comment = quote!{ #[doc=#text] };
                out.extend(doc_comment);
            },
            Self::DeriveCommon => {
                let derive = quote!{ #[derive(Debug, serde::Deserialize)] };
                out.extend(derive);
            },
            Self::DeriveNoSerde => {
                let derive = quote!{ #[derive(Debug)] };
                out.extend(derive);
            },
            Self::SerdeUntagged => {
                let derive = quote!{ #[serde(untagged)] };
                out.extend(derive);
            },
            Self::TypeAlias(type_alias, type_source) => {
                let type_alias = Ident::new(type_alias, Span::call_site());

                let type_source : syn::Type = syn::parse_str(type_source).expect("Unable to parse");
                let the_type = quote!{
                    pub type #type_alias = #type_source;
                };

                out.extend(the_type);
            },
            Self::Enum(enum_decl) => {
                let name = Ident::new(&enum_decl.name, Span::call_site());

                let variants = enum_decl.variants.iter()
                    .map(|variant| {
                        match variant {
                            EnumVariant::Tuple(e_name, e_type) => {
                                let e_name = Ident::new(&e_name, Span::call_site());
                                let e_type : syn::Type = syn::parse_str(e_type).expect("Unable to parse");
                                quote!{
                                    #e_name(#e_type)
                                }
                            },
                            EnumVariant::Unit(name, old_name) => {
                                let rename = match name == old_name {
                                    true => quote!{},
                                    false => quote!{ #[serde(rename=#old_name)]},
                                };
                                let name = Ident::new(&name, Span::call_site());
                                quote!{
                                    #rename
                                    #name
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                let the_enum = quote!{
                    pub enum #name {
                        #(#variants),*
                    }
                };

                out.extend(the_enum);
            },
            Self::TupleStruct(struct_name, struct_type) => {
                let struct_name = Ident::new(&struct_name, Span::call_site());
                let struct_type : syn::Type = syn::parse_str(struct_type).expect("Unable to parse");

                let tuple_struct = quote! {
                    pub struct #struct_name (#struct_type);
                };

                out.extend(tuple_struct);
            }
            Self::UnitStruct(struct_name) => {
                let struct_name = Ident::new(struct_name, Span::call_site());
                let unit_struct = quote!{
                    pub struct #struct_name;
                };

                out.extend(unit_struct);
            },
            Self::Struct(struct_decl) => {
                let struct_name = Ident::new(&struct_decl.name, Span::call_site());
                let fields = struct_decl.fields.iter()
                    .map(|field| {
                        let name = Ident::new(&field.name, Span::call_site());

                        let is_collection = field.field_type.starts_with("Vec");

                        let field_type : std::borrow::Cow<str> = match field.boxed {
                            true => format!("Box<{}>", field.field_type).into(),
                            false => (&field.field_type).into(),
                        };
                
                        let field_type = match field.required {
                            true => field_type,
                            false => {
                                if is_collection {
                                    field_type
                                } else {
                                    format!("Option<{}>", field_type).into()
                                }
                            }
                        };

                        let field_type : syn::Type = syn::parse_str(&field_type).expect(&format!("Unable to parse: {}", field_type));

                        let serde_default = match is_collection && !field.required {
                            true => quote!{ #[serde(default)] },
                            false => quote!{}
                        };

                        let rename = match field.name == field.old_name {
                            true => quote!{},
                            false => {
                                let old_name = &field.old_name;
                                quote!{ #[serde(rename=#old_name)]}
                            },
                        };

                        let description = match field.description.as_ref() {
                            Some(description) => quote!{#[doc=#description]},
                            None => quote!{},
                        };

                        quote!{
                            #rename
                            #serde_default
                            #description
                            pub #name: #field_type
                        }
                    })
                    .collect::<Vec<_>>();

                let deny = match struct_decl.additional_fields {
                    false => quote! { #[serde(deny_unknown_fields)] },
                    true => quote! {},
                };

                let the_struct = quote!{
                    #deny
                    pub struct #struct_name {
                        #(#fields),*
                    }
                };

                out.extend(the_struct);
            },
            Self::NumericValidator(name, config) => {
                let name = format_ident!("{}", name);
                let validator = quote!{
                    impl<'de> serde::Deserialize<'de> for #name {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: serde::Deserializer<'de>
                        {
                            let config = #config;

                            deserializer.deserialize_any(config).map(|n| Self(n))
                        }
                    }
                };

                out.extend(validator);
            },
            Self::StringValidator(name, config) => {
                let name = format_ident!("{}", name);
                let validator = quote!{
                    impl<'de> serde::Deserialize<'de> for #name {
                        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where
                            D: serde::Deserializer<'de>
                        {
                            let config = #config;

                            deserializer.deserialize_any(config).map(|n| Self(n))
                        }
                    }
                };

                out.extend(validator);
            },
        }
    }
}
