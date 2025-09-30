use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;

use super::*;

pub fn gen_code(comps: &[Component]) {
    let comp_enum = gen_component_enum(comps);
    let comp_type_enum = gen_component_type_enum(comps);
    let comp_get_type = gen_component_get_type(comps);
    let comp_type_id = gen_component_type_id(comps);
    let gen_comps = gen_components(comps);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY
        // Generated from components.toml by build.rs

        use std::ops::{Add, Sub};
        use borsh::{BorshSerialize, BorshDeserialize};
        use crate::Averageable;

        #comp_enum

        #comp_type_enum

        #comp_get_type

        #comp_type_id

        #gen_comps
    };

    // reconstruct, format, and save
    let syntax_tree = syn::parse2::<syn::File>(code).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // save to target/ dir
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("components.rs");
    fs::write(&out_path, formatted).expect("Failed to write components.rs");
}

fn gen_component_enum(comps: &[Component]) -> TokenStream {
    let variants: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                #name(#name)
            }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        pub enum Component {
            #(#variants),*
        }
    }
}

fn gen_component_type_enum(comps: &[Component]) -> TokenStream {
    let variants: Vec<_> = comps
        .iter()
        .map(|c| {
            let name = ident(&c.name);
            quote! { #name }
        })
        .collect();

    quote! {
        #[derive(Debug, PartialEq, Eq, Clone, Hash, BorshSerialize, BorshDeserialize)]
        pub enum ComponentType {
            #(#variants),*
        }
    }
}

fn gen_component_get_type(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|c| {
            let name = ident(&c.name);
            quote! {
                Component::#name(_) => ComponentType::#name
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn get_type(&self) -> ComponentType {
                match self {
                    #(#arms),*
                }
            }
        }
    }
}

fn gen_component_type_id(comps: &[Component]) -> TokenStream {
    let mut ctarms = Vec::with_capacity(comps.len());
    let mut carms = Vec::with_capacity(comps.len());

    for comp in comps {
        let name = ident(&comp.name);
        let id = comp.id;

        ctarms.push(quote! {
            ComponentType::#name => #id
        });

        carms.push(quote! {
            Component::#name(..) => #id
        });
    }

    quote! {
        impl ComponentType {
            pub fn get_id(&self) -> u16 {
                match self {
                    #(#ctarms),*
                }
            }
        }

        impl Component {
            pub fn get_id(&self) -> u16 {
                match self {
                    #(#carms),*
                }
            }
        }
    }
}

fn gen_components(comps: &[Component]) -> TokenStream {
    let comps: Vec<_> = comps
        .iter()
        .map(|comp| {
            let doc = gen_doc_attr(&comp.desc, &comp.related);
            let comp_code = match &comp.kind {
                ComponentKind::Single { field, .. } => gen_single_component(comp, field),
                ComponentKind::Struct { fields } => gen_struct_component(comp, fields),
                ComponentKind::Enum {
                    variants,
                    allow_custom,
                } => gen_enum_component(comp, variants, *allow_custom),
                ComponentKind::Marker => gen_marker_component(comp),
            };
            quote! {
                #doc
                #comp_code
            }
        })
        .collect();

    quote! { #(#comps)* }
}

fn gen_doc_attr(desc: &str, related: &[Related]) -> TokenStream {
    let mut doc_parts = Vec::new();

    if !desc.is_empty() {
        doc_parts.push(desc.to_string());
    }

    if !related.is_empty() {
        if !doc_parts.is_empty() {
            doc_parts.push(String::new());
        }
        doc_parts.push("Usually paired with:".to_string());

        for rel in related {
            doc_parts.push(format!(" - [{}] {}", rel.name, rel.reason));
        }
    }

    if doc_parts.is_empty() {
        quote! {}
    } else {
        let combined_doc = doc_parts.join("\n");
        quote! { #[doc = #combined_doc] }
    }
}

fn gen_marker_component(comp: &Component) -> TokenStream {
    let name = ident(&comp.name);

    quote! {
        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        pub struct #name;
    }
}

fn gen_single_component(comp: &Component, field: &str) -> TokenStream {
    let name = ident(&comp.name);
    let field_type = field.parse::<TokenStream>().unwrap_or_else(|_| {
        quote! { #field }
    });

    let averageable_impl = gen_single_averageable_impl(&comp.name, field);

    quote! {
        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        pub struct #name(pub #field_type);

        #averageable_impl
    }
}

fn gen_struct_component(comp: &Component, fields: &[Field]) -> TokenStream {
    let name = ident(&comp.name);

    let field_defs: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_name = ident(&field.name);
            let field_type = field
                .r#type
                .parse::<TokenStream>()
                .unwrap_or_else(|_| panic!("Failed to parse type: {}", field.r#type));
            quote! { pub #field_name: #field_type }
        })
        .collect();

    let averageable_impl = gen_struct_averageable_impl(comp, fields);

    quote! {
        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        pub struct #name {
            #(#field_defs),*
        }

        #averageable_impl
    }
}

/// If all fields in this struct are averageable (numbers, booleans)
/// Then we will create a struct to sum up all of this struct
/// And implement averageable for it
fn gen_struct_averageable_impl(comp: &Component, fields: &[Field]) -> TokenStream {
    let mut sum_types = Vec::new();
    for field in fields {
        if let Some(sum_type_str) = get_sum_type(&field.r#type) {
            sum_types.push((field, sum_type_str));
        } else {
            return quote! {}; // not Averageable -> dont impl
        }
    }

    if sum_types.is_empty() {
        return quote! {};
    }

    let name = ident(&comp.name);
    let sum_name = format_ident!("{}Sum", comp.name);

    let mut sum_fields = Vec::new();
    let mut add_fields = Vec::new();
    let mut sub_fields = Vec::new();
    let mut to_sum_fields = Vec::new();
    let mut from_sum_fields = Vec::new();

    for (field, sum_type_str) in &sum_types {
        let field_name = ident(&field.name);
        let field_type: TokenStream = field.r#type.parse().unwrap();
        let sum_type: TokenStream = sum_type_str.parse().unwrap();

        sum_fields.push(quote! { pub #field_name: #sum_type });
        add_fields.push(quote! { #field_name: self.#field_name + other.#field_name });
        sub_fields.push(quote! { #field_name: self.#field_name - other.#field_name });
        to_sum_fields.push(quote! { #field_name: self.#field_name as #sum_type });
        from_sum_fields.push(if field.r#type == "bool" {
            quote! { #field_name: (sum.#field_name / len as #sum_type) != 0 }
        } else if *sum_type_str != field.r#type {
            quote! { #field_name: (sum.#field_name / len as #sum_type) as #field_type }
        } else {
            quote! { #field_name: sum.#field_name / len as #sum_type }
        });
    }

    quote! {
        #[derive(Clone, Debug, Default, BorshSerialize, BorshDeserialize)]
        pub struct #sum_name {
            #(#sum_fields),*
        }

        impl Add for #sum_name {
            type Output = Self;
            fn add(self, other: Self) -> Self {
                #sum_name {
                    #(#add_fields),*
                }
            }
        }

        impl Sub for #sum_name {
            type Output = Self;
            fn sub(self, other: Self) -> Self {
                #sum_name {
                    #(#sub_fields),*
                }
            }
        }

        impl Averageable for #name {
            type Sum = #sum_name;

            fn to_sum_component(&self) -> Self::Sum {
                #sum_name {
                    #(#to_sum_fields),*
                }
            }

            fn from_sum(sum: Self::Sum, len: usize) -> Self {
                #name {
                    #(#from_sum_fields),*
                }
            }
        }
    }
}

fn gen_enum_component(comp: &Component, variants: &[Variant], allow_custom: bool) -> TokenStream {
    validate_enum_ids(&comp.name, variants);

    let name = ident(&comp.name);

    let mut sorted_variants = variants.to_vec();
    sorted_variants.sort_by_key(|v| v.id);

    let mut variant_defs: Vec<_> = sorted_variants
        .iter()
        .map(|v| {
            let variant = ident(&v.name);
            quote! {
                #variant
            }
        })
        .collect();

    let arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant = ident(&v.name);
            let variant_str = v.name.clone();
            let aliases = v.aliases.clone().unwrap_or_default();

            let pattern = if aliases.is_empty() {
                quote! { #variant_str }
            } else {
                let alias_patterns: Vec<_> = aliases.iter().map(|a| quote! { #a }).collect();
                quote! { #variant_str | #(#alias_patterns)|* }
            };

            quote! {
                #pattern => #name::#variant
            }
        })
        .collect();

    let display_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant = ident(&v.name);
            let variant_str = &v.name;
            quote! {
                #name::#variant => write!(f, "{}", #variant_str)
            }
        })
        .collect();

    let (from_impl, display_impl) = match allow_custom {
        true => {
            variant_defs.push(quote! {
                Custom(String) = 255
            });

            let from = quote! {
                impl From<String> for #name {
                    fn from(s: String) -> Self {
                        match s.as_str() {
                            #(#arms),*,
                            s => #name::Custom(s.to_string())
                        }
                    }
                }
            };

            let display = quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#display_arms),*,
                            #name::Custom(s) => write!(f, "{}", s)
                        }
                    }
                }
            };

            (from, display)
        }
        false => {
            let from = quote! {
                impl TryFrom<String> for #name {
                    type Error = ();

                    fn try_from(s: String) -> Result<Self, Self::Error> {
                        Ok(match s.as_str() {
                            #(#arms),*,
                            _ => return Err(())
                        })
                    }
                }
            };

            let display = quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        match self {
                            #(#display_arms),*
                        }
                    }
                }
            };

            (from, display)
        }
    };

    quote! {
        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        #[borsh(use_discriminant = true)]
        #[repr(u8)]
        pub enum #name {
            #(#variant_defs),*
        }

        #from_impl
        #display_impl
    }
}

fn validate_enum_ids(name: &str, variants: &[Variant]) {
    let mut ids = HashSet::new();

    for variant in variants {
        if !ids.insert(variant.id) {
            panic!(
                "{}::{} tried to use ID {} but it's already taken! Please take extreme caution to make sure IDs are consistent with old versions",
                name, variant.name, variant.id
            );
        }
    }

    if let (Some(&min), Some(&max)) = (ids.iter().min(), ids.iter().max()) {
        for id in min..=max {
            if !ids.contains(&id) {
                panic!("ID {} was skipped in {}!", id, name);
            }
        }
    }
}

/// If a Single kind has a type that is averageble (numbers, boolean)
/// Then we will impl the Averageable trait
fn gen_single_averageable_impl(name: &str, field_type: &str) -> TokenStream {
    let Some(sum_type) = get_sum_type(field_type) else {
        return quote! {};
    };

    let struct_name = ident(name);
    let sum_type_tokens: TokenStream = sum_type.parse().unwrap();
    let field_type_tokens: TokenStream = field_type.parse().unwrap();

    let from_sum_body = if field_type == "bool" {
        quote! { Self((sum / len as #sum_type_tokens) != 0) }
    } else if sum_type != field_type {
        quote! { Self((sum / len as #sum_type_tokens) as #field_type_tokens) }
    } else {
        quote! { Self(sum / len as #sum_type_tokens) }
    };

    quote! {
        impl Averageable for #struct_name {
            type Sum = #sum_type_tokens;

            fn to_sum_component(&self) -> Self::Sum {
                self.0 as Self::Sum
            }

            fn from_sum(sum: Self::Sum, len: usize) -> Self
            where
                Self: Sized,
            {
                #from_sum_body
            }
        }
    }
}

/// returns what type to sum with when averaging
/// IE we can sum up u8's with u8's because it will overflow
fn get_sum_type(field_type: &str) -> Option<&'static str> {
    match field_type {
        "u8" | "u16" | "u32" => Some("u64"),
        "u64" => Some("u64"),
        "u128" => Some("u128"),
        "i8" | "i16" | "i32" => Some("i64"),
        "i64" => Some("i64"),
        "i128" => Some("i128"),
        "f32" => Some("f64"),
        "f64" => Some("f64"),
        "bool" => Some("u32"),
        _ => None,
    }
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
