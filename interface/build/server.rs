use crate::rust::{comp_name_to_cmd_name, ident, upper_camel_to_snake};

use super::model::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{env, fs, path::PathBuf};

pub fn generate(_cmds: &[Command], comps: &[Component]) {
    let comp_enum = gen_comp_enum(comps);
    let primitives_avgable = gen_primitives_avgable();
    let structs_avgable = gen_structs_avgable(comps);
    let avg_comp = gen_avg_comp(comps);
    let read_comp = gen_read_comp(comps);
    let write_comp = gen_write_comp(comps);
    let comp_inner = gen_comp_inner(comps);
    let comp_from_string = gen_comp_from_string(comps);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY

        use std::ops::{Add, Sub};

        #comp_enum

        #primitives_avgable

        #structs_avgable

        #avg_comp

        #read_comp

        #write_comp

        #comp_inner

        #comp_from_string
    };

    // reconstruct, format, and save
    let syntax_tree = syn::parse2::<syn::File>(code).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // save to target/ dir
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("server.rs");
    fs::write(&out_path, formatted).expect("Failed to write server.rs");
}

fn gen_comp_from_string(comps: &[Component]) -> TokenStream {
    let parse_error = quote! {
        #[derive(Debug, Clone)]
        pub enum ParseError {
            InvalidValue,
            UnsupportedType,
        }
    };

    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let comp_name = ident(&comp.name);

            match &comp.kind {
                ComponentKind::Single { field, .. } if !field.contains("Vec<") => {
                    gen_single_arm(&comp.name, field)
                }
                ComponentKind::Enum { variants } => gen_enum_arm(&comp.name, variants),
                _ => {
                    quote! {
                        ComponentType::#comp_name => Err(ParseError::UnsupportedType)
                    }
                }
            }
        })
        .collect();

    quote! {
        #parse_error

        impl Component {
            pub fn from_string(comp_type: ComponentType, s: &str) -> Result<Component, ParseError> {
                match comp_type {
                    #(#arms,)*
                }
            }
        }
    }
}

fn gen_single_arm(comp_name: &str, field: &str) -> TokenStream {
    let comp_ident = ident(comp_name);
    if field == "String" {
        quote! {
            ComponentType::#comp_ident => Ok(Component::#comp_ident(s.to_string()))
        }
    } else {
        let field_type: TokenStream = field.parse().unwrap();
        quote! {
            ComponentType::#comp_ident => {
                s.parse::<#field_type>()
                    .map(Component::#comp_ident)
                    .map_err(|_| ParseError::InvalidValue)
            }
        }
    }
}

fn gen_enum_arm(comp_name: &str, variants: &[Variant]) -> TokenStream {
    let comp_ident = ident(comp_name);

    let variant_arms: Vec<_> = variants
        .iter()
        .map(|v| {
            let variant_ident = ident(&v.name);
            let id = v.id;
            quote! {
                #id => Ok(Component::#comp_ident(#comp_ident::#variant_ident))
            }
        })
        .collect();

    quote! {
        ComponentType::#comp_ident => {
            let discriminant = s.parse::<u8>()
                .map_err(|_| ParseError::InvalidValue)?;
            match discriminant {
                #(#variant_arms,)*
                _ => Err(ParseError::InvalidValue)
            }
        }
    }
}

fn gen_comp_inner(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            if comp.is_marker() {
                quote! {}
            } else {
                quote! {
                    Component::#name(payload) => {
                        Some(format!("{payload:?}"))
                    }
                }
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn inner_string(&self) -> Option<String> {
                match self {
                    #(#arms)*
                    _ => None
                }
            }
        }
    }
}

fn gen_write_comp(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let func_name = ident(&upper_camel_to_snake(&comp.name));
            let name = ident(&comp.name);

            if comp.is_marker() {
                quote! {
                    Component::#name => {
                        self.#func_name().await
                    }
                }
            } else {
                quote! {
                    Component::#name(payload) => {
                        self.#func_name(payload).await
                    }
                }
            }
        })
        .collect();

    quote! {
        #[cfg(feature = "floe")]
        impl<W: AsyncWriteExt + Unpin> FloeWriter<W> {
            pub async fn write_component(&mut self, comp: &Component) -> Result<(), std::io::Error> {
                match comp {
                    #(#arms)*
                }
            }
        }
    }
}

fn gen_read_comp(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let cmd_name = ident(&comp_name_to_cmd_name(&comp.name));
            let name = ident(&comp.name);

            if comp.is_marker() {
                quote! {
                    #cmd_name => {
                        Ok(Component::#name)
                    }
                }
            } else {
                quote! {
                    #cmd_name => {
                        Ok(Component::#name(borsh::from_slice(&payload)?))
                    }
                }
            }
        })
        .collect();

    quote! {
        pub fn read_component(cmd_id: u16, payload: Vec<u8>) -> Result<Component, std::io::Error> {
            match cmd_id {
                #(#arms)*
                _ => unreachable!()
            }
        }
    }
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
        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        #[repr(u16)]
        #[borsh(use_discriminant=false)]
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

fn gen_avg_comp(comps: &[Component]) -> TokenStream {
    let variants: Vec<_> = comps
        .iter()
        .filter(|comp| is_avgable(comp))
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                #name(Average<#name>)
            }
        })
        .collect();

    let new_arms: Vec<_> = comps
        .iter()
        .filter(|comp| is_avgable(comp))
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                ComponentType::#name => Some(ComponentAverage::#name(Average::new()))
            }
        })
        .collect();

    let add_arms: Vec<_> = comps
        .iter()
        .filter(|comp| is_avgable(comp))
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                ComponentAverage::#name(avg) => {
                    let Component::#name(v) = comp else {
                        panic!("Type mismatch in ComponentAverage::add: expected {}, got {:?}", stringify!(#name), comp);
                    };
                    avg.add(v);
                }
            }
        })
        .collect();

    let remove_arms: Vec<_> = comps
        .iter()
        .filter(|comp| is_avgable(comp))
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                ComponentAverage::#name(avg) => {
                    let Component::#name(v) = comp else {
                        panic!("Type mismatch in ComponentAverage::remove: expected {}, got {:?}", stringify!(#name), comp);
                    };
                    avg.remove(v);
                }
            }
        })
        .collect();

    let update_arms: Vec<_> = comps
        .iter()
        .filter(|comp| is_avgable(comp))
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                ComponentAverage::#name(avg) => {
                    let Component::#name(old_v) = old else {
                        panic!("Type mismatch in ComponentAverage::update old: expected {}, got {:?}", stringify!(#name), old);
                    };
                    let Component::#name(new_v) = new else {
                        panic!("Type mismatch in ComponentAverage::update new: expected {}, got {:?}", stringify!(#name), new);
                    };
                    avg.update(old_v, new_v);
                }
            }
        })
        .collect();

    let current_average_arms: Vec<_> = comps
        .iter()
        .filter(|comp| is_avgable(comp))
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                ComponentAverage::#name(avg) => avg.current_average().map(Component::#name)
            }
        })
        .collect();

    let len_arms: Vec<_> = comps
        .iter()
        .filter(|comp| is_avgable(comp))
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                ComponentAverage::#name(avg) => avg.len()
            }
        })
        .collect();

    let is_empty_arms: Vec<_> = comps
        .iter()
        .filter(|comp| is_avgable(comp))
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                ComponentAverage::#name(avg) => avg.is_empty()
            }
        })
        .collect();

    quote! {
        #[derive(Clone, Debug)]
        pub enum ComponentAverage {
            #(#variants,)*
        }

        impl ComponentAverage {
            /// None if comp type is not averageable
            pub fn new(comp_type: ComponentType) -> Option<Self> {
                match comp_type {
                    #(#new_arms,)*
                    _ => None,
                }
            }

            /// panics if the comp type != averages type
            pub fn add(&mut self, comp: Component) {
                match self {
                    #(#add_arms,)*
                }
            }

            /// panics if the comp type != averages type
            pub fn remove(&mut self, comp: Component) {
                match self {
                    #(#remove_arms,)*
                }
            }

            /// panics if the comp type != averages type
            pub fn update(&mut self, old: Component, new: Component) {
                match self {
                    #(#update_arms,)*
                }
            }

            /// None if no components have been added
            pub fn current_average(&self) -> Option<Component> {
                match self {
                    #(#current_average_arms,)*
                }
            }

            pub fn len(&self) -> usize {
                match self {
                    #(#len_arms,)*
                }
            }

            pub fn is_empty(&self) -> bool {
                match self {
                    #(#is_empty_arms,)*
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

fn is_avgable(comp: &Component) -> bool {
    match &comp.kind {
        ComponentKind::Single { field, .. } => AVGABLE_PRIMS.contains(&field.as_str()),
        ComponentKind::Struct { fields } => is_struct_avgable(fields),
        _ => false,
    }
}

/// If all fields in this struct are averageable (numbers, booleans)
fn is_struct_avgable(fields: &[Field]) -> bool {
    let mut sum_types = Vec::new();
    for field in fields {
        if let Some(sum_type_str) = get_sum_type(&field.r#type) {
            sum_types.push((field, sum_type_str));
        } else {
            return false;
        }
    }
    !sum_types.is_empty()
}

fn gen_struct_avgable(comp: &Component, fields: &[Field]) -> TokenStream {
    if !is_struct_avgable(fields) {
        return quote![];
    }

    let mut sum_types = Vec::new();
    for field in fields {
        let sum_type_str = get_sum_type(&field.r#type).unwrap();
        sum_types.push((field, sum_type_str));
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

const AVGABLE_PRIMS: [&str; 13] = [
    "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "bool",
];

fn gen_primitives_avgable() -> TokenStream {
    let impls: Vec<_> = AVGABLE_PRIMS
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
