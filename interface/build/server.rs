use super::model::*;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::{env, fs, path::PathBuf};
use syn::Ident;

pub fn generate(_cmds: &[Command], comps: &[Component]) {
    let comp_enum = gen_comp_enum(comps);
    let primitives_avgable = gen_primitives_avgable();
    let structs_avgable = gen_structs_avgable(comps);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY

        use std::ops::{Add, Sub};
        use crate::avg::Averageable;

        #comp_enum

        #primitives_avgable

        #structs_avgable
    };

    // reconstruct, format, and save
    let syntax_tree = syn::parse2::<syn::File>(code).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // save to target/ dir
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("server.rs");
    fs::write(&out_path, formatted).expect("Failed to write server.rs");
}

fn gen_comp_enum(comps: &[Component]) -> TokenStream {
    let enum_variants = comps.iter().map(|comp| {
        let name = ident(&comp.name);
        let id = comp.id;
        if comp.is_marker() {
            quote! { #name = #id }
        } else {
            quote! { #name(#name) = #id }
        }
    });

    let get_type_arms = comps.iter().map(|comp| {
        let name = ident(&comp.name);
        if comp.is_marker() {
            quote! { Component::#name => ComponentType::#name }
        } else {
            quote! { Component::#name(_) => ComponentType::#name }
        }
    });

    let get_type_id_arms = comps.iter().map(|comp| {
        let name = ident(&comp.name);
        let id = comp.id;
        if comp.is_marker() {
            quote! { Component::#name => #id }
        } else {
            quote! { Component::#name(_) => #id }
        }
    });

    quote! {
        #[derive(Debug, Clone, PartialEq)]
        #[repr(u16)]
        pub enum Component {
            #(#enum_variants),*
        }

        impl Component {
            pub fn get_type(&self) -> ComponentType {
                match self {
                    #(#get_type_arms),*
                }
            }

            pub fn get_type_id(&self) -> u16 {
                match self {
                    #(#get_type_id_arms),*
                }
            }
        }
    }
}
fn gen_structs_avgable(comps: &[Component]) -> TokenStream {
    let impls: Vec<_> = comps
        .iter()
        .map(|comp| {
            if let ComponentKind::Struct { fields } = &comp.kind {
                gen_struct_avgable(comp, fields)
            } else {
                quote! {}
            }
        })
        .collect();

    quote! {
        #(#impls)*
    }
}

/// If all fields in this struct are averageable (numbers, booleans)
/// Then we will create a struct to sum up all of this struct
/// And implement averageable for it
fn gen_struct_avgable(comp: &Component, fields: &[Field]) -> TokenStream {
    let mut sum_types = Vec::new();
    for field in fields {
        if let Some(sum_type_str) = get_sum_type(&field.r#type) {
            sum_types.push((field, sum_type_str));
        } else {
            return quote! {}; // not avgable
        }
    }

    if sum_types.is_empty() {
        return quote! {};
    }

    let name = ident(&comp.name);
    let sum_name = format_ident!("{}Sum", comp.name);

    // FIXME my eyes hurt

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

            fn to_sum_repr(self) -> Self::Sum {
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

fn gen_primitives_avgable() -> TokenStream {
    let typs = [
        "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "bool",
    ];

    let impls: Vec<_> = typs
        .iter()
        .map(|typ| gen_primitive_averageable(typ))
        .collect();

    quote! {
        #(#impls)*
    }
}

fn gen_primitive_averageable(typ_str: &str) -> TokenStream {
    let name = ident(typ_str);
    let sum_type_str = get_sum_type(typ_str).unwrap();

    let sum_type: TokenStream = sum_type_str.parse().unwrap();
    let typ: TokenStream = typ_str.parse().unwrap();

    let from_sum_body = if typ_str == "bool" {
        quote! { (sum / len as #sum_type) != 0 }
    } else if sum_type_str != typ_str {
        quote! { (sum / len as #sum_type) as #typ }
    } else {
        quote! { sum / len as #sum_type }
    };

    quote! {
        impl Averageable for #name {
            type Sum = #sum_type;

            fn to_sum_repr(self) -> Self::Sum {
                self as Self::Sum
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
        "u8" | "u16" | "u32" | "u64" => Some("u64"),
        "u128" => Some("u128"),
        "i8" | "i16" | "i32" | "i64" => Some("i64"),
        "i128" => Some("i128"),
        "f32" | "f64" => Some("f64"),
        "bool" => Some("u32"),
        _ => None,
    }
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
