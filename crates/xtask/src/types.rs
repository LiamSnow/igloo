use super::model::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

pub fn gen_str_funcs(comps: &[Component]) -> TokenStream {
    let snakes: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            let snake = upper_camel_to_snake(&comp.name);
            quote! {
                ComponentType::#name => #snake
            }
        })
        .collect();

    let kebabs: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            let kebab = upper_camel_to_kebab(&comp.name);
            quote! {
                ComponentType::#name => #kebab
            }
        })
        .collect();

    quote! {
        impl ComponentType {
            pub fn snake_name(&self) -> &'static str {
                match self {
                    #(#snakes,)*
                }
            }

            pub fn kebab_name(&self) -> &'static str {
                match self {
                    #(#kebabs,)*
                }
            }
        }
    }
}

pub fn gen_comp_igloo_type(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);

            match &comp.kind {
                ComponentKind::Single { kind } => {
                    let igloo_type = kind.tokens();
                    quote! {
                        ComponentType::#name => Some(IglooType::#igloo_type)
                    }
                }
                ComponentKind::Enum { .. } => {
                    quote! {
                        ComponentType::#name => Some(IglooType::Enum(IglooEnumType::#name))
                    }
                }
                ComponentKind::Marker { .. } => {
                    quote! {
                        ComponentType::#name => None
                    }
                }
            }
        })
        .collect();

    quote! {
        impl ComponentType {
            pub fn igloo_type(&self) -> Option<IglooType> {
                match self {
                    #(#arms,)*
                }
            }
        }
    }
}

pub fn gen_comp_type(comps: &[Component]) -> TokenStream {
    let variants: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            quote! { #name }
        })
        .collect();

    quote! {
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
        #[repr(u16)]
        pub enum ComponentType {
            #(#variants),*
        }
    }
}

pub fn upper_camel_to_snake(s: &str) -> String {
    let mut res = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            res.push('_');
        }
        res.push(c.to_ascii_lowercase());
    }
    res
}

pub fn upper_camel_to_kebab(s: &str) -> String {
    let mut res = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            res.push('-');
        }
        res.push(c.to_ascii_lowercase());
    }
    res
}

pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
