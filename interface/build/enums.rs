use super::model::*;
use super::types::ident;
use proc_macro2::TokenStream;
use quote::quote;

pub fn gen_enum_types(comps: &[Component]) -> TokenStream {
    let enum_comps: Vec<_> = comps
        .iter()
        .filter(|comp| matches!(comp.kind, ComponentKind::Enum { .. }))
        .collect();
    let count = enum_comps.len();

    let type_variants: Vec<_> = enum_comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            quote! { #name }
        })
        .collect();

    let value_variants: Vec<_> = enum_comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            quote! { #name(#name) }
        })
        .collect();

    let from_string_arms: Vec<_> = enum_comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                IglooEnumType::#name =>
                    #name::try_from(s).ok().map(IglooEnumValue::#name)
            }
        })
        .collect();

    let default_arms: Vec<_> = enum_comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            let ComponentKind::Enum { variants, .. } = &comp.kind else {
                unreachable!()
            };

            match variants.iter().next() {
                Some(first_var) => {
                    let first_var = ident(&first_var.name);
                    quote! {
                        IglooEnumType::#name
                            => IglooEnumValue::#name(#name::#first_var)
                    }
                }
                None => {
                    quote! {}
                }
            }
        })
        .collect();

    let get_type_arms: Vec<_> = enum_comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                IglooEnumValue::#name(_) => IglooEnumType::#name
            }
        })
        .collect();

    let display_arms: Vec<_> = enum_comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            quote! {
                IglooEnumValue::#name(v) => write!(f, "{}", v)
            }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum IglooEnumType {
            #(#type_variants),*
        }

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum IglooEnumValue {
            #(#value_variants),*
        }

        impl IglooEnumValue {
            pub fn from_string(enum_type: &IglooEnumType, s: String) -> Option<Self> {
                match enum_type {
                    #(#from_string_arms),*
                }
            }

            pub fn default(enum_type: &IglooEnumType) -> Self {
                match enum_type {
                    #(#default_arms),*
                }
            }

            pub fn get_type(&self) -> IglooEnumType {
                match self {
                    #(#get_type_arms),*
                }
            }
        }

        impl std::fmt::Display for IglooEnumValue {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#display_arms),*
                }
            }
        }

        pub static IGLOO_ENUMS: [IglooEnumType; #count] = [
            #(
                IglooEnumType::#type_variants
            ),*
        ];
    }
}
