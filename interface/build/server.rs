use super::model::*;
use crate::{agg::gen_aggregator, rust::{comp_name_to_cmd_name, ident, upper_camel_to_snake}};
use proc_macro2::TokenStream;
use quote::quote;
use std::{env, fs, path::PathBuf};

pub fn generate(_cmds: &[Command], comps: &[Component]) {
    let comp_enum = gen_comp_enum(comps);
    let read_comp = gen_read_comp(comps);
    let write_comp = gen_write_comp(comps);
    let comp_inner = gen_comp_inner(comps);
    let comp_from_string = gen_comp_from_string(comps);
    let to_igloo_value = gen_to_igloo_value(comps);
    let aggregator = gen_aggregator(comps);
    let from_igloo_value = gen_from_igloo_value(comps);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY
         
        use crate::types::agg::AggregationOp;
        use std::cmp::Ordering;

        #comp_enum

        #read_comp

        #write_comp

        #comp_inner

        #comp_from_string

        #to_igloo_value

        #from_igloo_value

        #aggregator
    };

    // reconstruct, format, and save
    let syntax_tree = syn::parse2::<syn::File>(code).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // save to target/ dir
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("server.rs");
    fs::write(&out_path, formatted).expect("Failed to write server.rs");
}

fn gen_to_igloo_value(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            
            match &comp.kind {
                ComponentKind::Single { kind } => {
                    let igloo_variant = kind.tokens();
                    quote! {
                        Component::#name(v) => Some(IglooValue::#igloo_variant(v.clone()))
                    }
                }
                ComponentKind::Enum { .. } => {
                    quote! {
                        Component::#name(v) => Some(IglooValue::Enum(IglooEnumValue::#name(v.clone())))
                    }
                }
                ComponentKind::Marker { .. } => {
                    quote! {
                        Component::#name => None
                    }
                }
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn to_igloo_value(&self) -> Option<IglooValue> {
                match self {
                    #(#arms,)*
                }
            }
        }
    }
}

fn gen_from_igloo_value(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            
            match &comp.kind {
                ComponentKind::Single { kind } => {
                    let igloo_variant = kind.tokens();
                    quote! {
                        ComponentType::#name => {
                            if let IglooValue::#igloo_variant(v) = value {
                                Some(Component::#name(v))
                            } else {
                                None
                            }
                        }
                    }
                }
                ComponentKind::Enum { .. } => {
                    quote! {
                        ComponentType::#name => {
                            if let IglooValue::Enum(IglooEnumValue::#name(v)) = value {
                                Some(Component::#name(v))
                            } else {
                                None
                            }
                        }
                    }
                }
                ComponentKind::Marker { .. } => {
                    quote! {
                        ComponentType::#name => Some(Component::#name)
                    }
                }
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn from_igloo_value(r#type: ComponentType, value: IglooValue) -> Option<Self> {
                match r#type {
                    #(#arms,)*
                }
            }
        }
    }
}

fn gen_comp_from_string(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .filter_map(|comp| {
            let comp_ident = ident(&comp.name);

            match &comp.kind {
                ComponentKind::Single { kind } => {
                    let parse_logic = match kind {
                        IglooType::Integer | IglooType::Real | IglooType::Boolean
                        | IglooType::Color | IglooType::Date | IglooType::Time => {
                            quote! { s.parse().ok().map(Component::#comp_ident) }
                        }
                        IglooType::Text => {
                            quote! { Some(Component::#comp_ident(s)) }
                        }
                        IglooType::IntegerList | IglooType::RealList | IglooType::BooleanList
                        | IglooType::ColorList | IglooType::DateList | IglooType::TimeList => {
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
        #[cfg_attr(feature = "penguin", derive(serde::Serialize, serde::Deserialize))]
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


