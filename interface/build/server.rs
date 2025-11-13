use super::model::*;
use crate::rust::{comp_name_to_cmd_name, ident, upper_camel_to_snake};
use proc_macro2::TokenStream;
use quote::quote;
use std::{env, fs, path::PathBuf};

pub fn generate(_cmds: &[Command], comps: &[Component]) {
    let comp_enum = gen_comp_enum(comps);
    let read_comp = gen_read_comp(comps);
    let write_comp = gen_write_comp(comps);
    let comp_inner = gen_comp_inner(comps);
    let comp_from_string = gen_comp_from_string(comps);
    let enum_aggregatable = gen_enum_aggregatable(comps);
    let component_aggregate = gen_component_aggregate(comps);
    let to_igloo_value = gen_to_igloo_value(comps);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY

        use crate::types::agg::{Aggregatable, AggregationOp};

        #comp_enum

        #read_comp

        #write_comp

        #comp_inner

        #comp_from_string

        #enum_aggregatable

        #component_aggregate

        #to_igloo_value
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
                    let igloo_variant = match kind {
                        IglooType::Integer => quote! { IglooValue::Integer },
                        IglooType::Real => quote! { IglooValue::Real },
                        IglooType::Text => quote! { IglooValue::Text },
                        IglooType::Boolean => quote! { IglooValue::Boolean },
                        IglooType::Color => quote! { IglooValue::Color },
                        IglooType::Date => quote! { IglooValue::Date },
                        IglooType::Time => quote! { IglooValue::Time },
                        IglooType::IntegerList => quote! { IglooValue::IntegerList },
                        IglooType::RealList => quote! { IglooValue::RealList },
                        IglooType::TextList => quote! { IglooValue::TextList },
                        IglooType::BooleanList => quote! { IglooValue::BooleanList },
                        IglooType::ColorList => quote! { IglooValue::ColorList },
                        IglooType::DateList => quote! { IglooValue::DateList },
                        IglooType::TimeList => quote! { IglooValue::TimeList },
                    };
                    quote! {
                        Component::#name(v) => Some(#igloo_variant(v.clone()))
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


fn gen_enum_aggregatable(comps: &[Component]) -> TokenStream {
    let enum_comps: Vec<_> = comps
        .iter()
        .filter(|comp| matches!(comp.kind, ComponentKind::Enum { .. }))
        .collect();

    let impls: Vec<_> = enum_comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            
            if let ComponentKind::Enum { variants, .. } = &comp.kind {
                let variant_count = variants.len();
                let variant_arms: Vec<_> = variants
                    .iter()
                    .enumerate()
                    .map(|(idx, v)| {
                        let variant_name = ident(&v.name);
                        quote! {
                            #name::#variant_name => #idx
                        }
                    })
                    .collect();

                let from_idx_arms: Vec<_> = variants
                    .iter()
                    .enumerate()
                    .map(|(idx, v)| {
                        let variant_name = ident(&v.name);
                        quote! {
                            #idx => #name::#variant_name
                        }
                    })
                    .collect();

                quote! {
                    impl Aggregatable for #name {
                        fn aggregate<I: IntoIterator<Item = Self>>(items: I, op: AggregationOp) -> Option<Self> {
                            match op {
                                AggregationOp::Mean => {
                                    let mut counts = [0usize; #variant_count];
                                    let mut total = 0;
                                    for item in items {
                                        let idx = match item {
                                            #(#variant_arms,)*
                                        };
                                        counts[idx] += 1;
                                        total += 1;
                                    }
                                    
                                    if total == 0 {
                                        return None;
                                    }
                                    
                                    let mut max_idx = 0;
                                    for idx in 1..#variant_count {
                                        if counts[idx] > counts[max_idx] {
                                            max_idx = idx;
                                        }
                                    }                                    

                                    let result = match max_idx {
                                        #(#from_idx_arms,)*
                                        _ => unreachable!(),
                                    };
                                    Some(result)
                                }
                                _ => None,
                            }
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

fn gen_component_aggregate(comps: &[Component]) -> TokenStream {
    let aggregatable_comps: Vec<_> = comps
        .iter()
        .filter(|comp| is_aggregatable(comp))
        .collect();

    let arms: Vec<_> = aggregatable_comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            let name_str = &comp.name;
            
            quote! {
                Component::#name(_) => {
                    let iter = items.into_iter().map(|item| {
                        match item {
                            Component::#name(v) => v,
                            _ => panic!("Heterogeneous components in aggregate: expected {}", #name_str),
                        }
                    });
                    #name::aggregate(iter, op).map(Component::#name)
                }
            }
        })
        .collect();

    quote! {
        impl Component {
            pub fn aggregate(items: Vec<Component>, op: AggregationOp) -> Option<Component> {
                if items.is_empty() {
                    return None;
                }

                match &items[0] {
                    #(#arms,)*
                    _ => None,
                }
            }
        }
    }
}

fn is_aggregatable(comp: &Component) -> bool {
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
