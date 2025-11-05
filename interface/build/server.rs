use super::model::*;
use crate::rust::{comp_name_to_cmd_name, ident, upper_camel_to_snake};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{env, fs, path::PathBuf};

pub fn generate(_cmds: &[Command], comps: &[Component]) {
    let comp_enum = gen_comp_enum(comps);
    let enum_avgable = gen_enum_averageable(comps);
    let avg_comp = gen_avg_comp(comps);
    let read_comp = gen_read_comp(comps);
    let write_comp = gen_write_comp(comps);
    let comp_inner = gen_comp_inner(comps);
    let comp_from_string = gen_comp_from_string(comps);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY

        use std::ops::{Add, Sub};
        use crate::types::*;
        use crate::avg::*;

        #comp_enum

        #enum_avgable

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
    let arms: Vec<_> = comps
        .iter()
        .filter_map(|comp| {
            let comp_ident = ident(&comp.name);

            match &comp.kind {
                ComponentKind::Single { kind } => {
                    let parse_logic = match kind {
                        IglooType::Integer | IglooType::Real | IglooType::Boolean => {
                            quote! { s.parse().ok().map(Component::#comp_ident) }
                        }
                        IglooType::Text => {
                            quote! { Some(Component::#comp_ident(s)) }
                        }
                        IglooType::Color | IglooType::Date | IglooType::Time => {
                            quote! { s.try_into().ok().map(Component::#comp_ident) }
                        }
                        IglooType::IntegerList | IglooType::RealList | IglooType::BooleanList => {
                            quote! {
                                parse_list(&s)?
                                    .into_iter()
                                    .map(|item| item.parse().ok())
                                    .collect::<Option<Vec<_>>>()
                                    .map(Component::#comp_ident)
                            }
                        }
                        IglooType::TextList => {
                            quote! { parse_list(&s).map(Component::#comp_ident) }
                        }
                        IglooType::ColorList | IglooType::DateList | IglooType::TimeList => {
                            quote! {
                                parse_list(&s)?
                                    .into_iter()
                                    .map(|item| item.try_into().ok())
                                    .collect::<Option<Vec<_>>>()
                                    .map(Component::#comp_ident)
                            }
                        }
                    };

                    Some(quote! {
                        ComponentType::#comp_ident => #parse_logic
                    })
                }
                ComponentKind::Enum { .. } => Some(quote! {
                    ComponentType::#comp_ident => s.try_into().ok().map(Component::#comp_ident)
                }),
                ComponentKind::Marker { .. } => None,
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn from_string(comp_type: ComponentType, s: String) -> Option<Component> {
                match comp_type {
                    #(#arms,)*
                    _ => None
                }
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
        impl<W: AsyncWriteExt + Unpin> crate::floe::FloeWriter<W> {
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
            let name_str = &comp.name;
            quote! {
                ComponentAverage::#name(avg) => {
                    let Component::#name(v) = comp else {
                        panic!("Type mismatch in ComponentAverage::add: expected {}, got {:?}", #name_str, comp);
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
            let name_str = &comp.name;
            quote! {
                ComponentAverage::#name(avg) => {
                    let Component::#name(v) = comp else {
                        panic!("Type mismatch in ComponentAverage::remove: expected {}, got {:?}", #name_str, comp);
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
            let name_str = &comp.name;
            quote! {
                ComponentAverage::#name(avg) => {
                    let Component::#name(old_v) = old else {
                        panic!("Type mismatch in ComponentAverage::update old: expected {}, got {:?}", #name_str, old);
                    };
                    let Component::#name(new_v) = new else {
                        panic!("Type mismatch in ComponentAverage::update new: expected {}, got {:?}", #name_str, new);
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

fn is_avgable(comp: &Component) -> bool {
    match &comp.kind {
        ComponentKind::Single { kind, .. } => matches!(
            kind,
            IglooType::Integer
                | IglooType::Real
                | IglooType::Boolean
                | IglooType::Color
                | IglooType::Date
                | IglooType::Time
        ),
        ComponentKind::Enum { .. } => true,
        ComponentKind::Marker { .. } => false,
    }
}

fn gen_enum_averageable(comps: &[Component]) -> TokenStream {
    let enum_comps: Vec<_> = comps
        .iter()
        .filter(|comp| matches!(comp.kind, ComponentKind::Enum { .. }))
        .collect();

    let impls: Vec<_> = enum_comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            let sum_name = format_ident!("{}Sum", comp.name);

            if let ComponentKind::Enum { variants, .. } = &comp.kind {
                let sum_fields: Vec<_> = variants
                    .iter()
                    .map(|v| {
                        let field_name = ident(&upper_camel_to_snake(&v.name));
                        quote! { pub #field_name: usize }
                    })
                    .collect();

                let add_fields: Vec<_> = variants
                    .iter()
                    .map(|v| {
                        let field_name = ident(&upper_camel_to_snake(&v.name));
                        quote! { #field_name: self.#field_name + other.#field_name }
                    })
                    .collect();

                let sub_fields: Vec<_> = variants
                    .iter()
                    .map(|v| {
                        let field_name = ident(&upper_camel_to_snake(&v.name));
                        quote! { #field_name: self.#field_name - other.#field_name }
                    })
                    .collect();

                let to_sum_arms: Vec<_> = variants
                    .iter()
                    .map(|v| {
                        let variant_name = ident(&v.name);
                        let field_name = ident(&upper_camel_to_snake(&v.name));
                        quote! {
                            #name::#variant_name => {
                                sum.#field_name = 1;
                            }
                        }
                    })
                    .collect();

                let field_names: Vec<_> = variants
                    .iter()
                    .map(|v| ident(&upper_camel_to_snake(&v.name)))
                    .collect();

                let from_sum_checks: Vec<_> = variants
                    .iter()
                    .map(|v| {
                        let variant_name = ident(&v.name);
                        let field_name = ident(&upper_camel_to_snake(&v.name));
                        quote! {
                            if sum.#field_name >= max {
                                return #name::#variant_name;
                            }
                        }
                    })
                    .collect();

                let max_expr = if field_names.len() == 1 {
                    let f = &field_names[0];
                    quote! { sum.#f }
                } else {
                    let first = &field_names[0];
                    let mut expr = quote! { sum.#first };
                    for f in &field_names[1..] {
                        expr = quote! { #expr.max(sum.#f) };
                    }
                    expr
                };

                let first_variant = ident(&variants[0].name);

                quote! {
                    #[derive(Clone, Debug, Default)]
                    pub struct #sum_name {
                        #(#sum_fields),*
                    }

                    impl Add for #sum_name {
                        type Output = Self;
                        fn add(self, other: Self) -> Self {
                            Self {
                                #(#add_fields),*
                            }
                        }
                    }

                    impl Sub for #sum_name {
                        type Output = Self;
                        fn sub(self, other: Self) -> Self {
                            Self {
                                #(#sub_fields),*
                            }
                        }
                    }

                    impl Averageable for #name {
                        type Sum = #sum_name;

                        fn to_sum_repr(self) -> Self::Sum {
                            let mut sum = #sum_name::default();
                            match self {
                                #(#to_sum_arms)*
                            }
                            sum
                        }

                        fn from_sum(sum: Self::Sum, _len: usize) -> Self {
                            let max = #max_expr;
                            #(#from_sum_checks)*
                            #name::#first_variant
                        }
                    }
                }
            } else {
                unreachable!()
            }
        })
        .collect();

    quote! {
        #(#impls)*
    }
}
