use super::model::*;
use super::types::ident;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;

pub fn gen_comp_enum(comps: &[Component]) -> TokenStream {
    let enum_variants = comps.iter().map(|comp| {
        let name = ident(&comp.name);
        let id = comp.id;
        match &comp.kind {
            ComponentKind::Single { kind } => {
                let field_type = kind.direct_type_tokens();
                quote! { #name(#field_type) = #id }
            }
            ComponentKind::Enum { .. } => {
                quote! { #name(#name) = #id }
            }
            ComponentKind::Marker { .. } => quote! { #name = #id },
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
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

pub fn gen_enum_comps(comps: &[Component]) -> TokenStream {
    let enum_comps: Vec<_> = comps
        .iter()
        .filter(|comp| matches!(comp.kind, ComponentKind::Enum { .. }))
        .map(|comp| comp.gen_enum_def())
        .collect();

    quote! {
        #(#enum_comps)*
    }
}

impl Component {
    pub fn is_marker(&self) -> bool {
        matches!(self.kind, ComponentKind::Marker { .. })
    }

    fn gen_enum_def(&self) -> TokenStream {
        if let ComponentKind::Enum { variants, .. } = &self.kind {
            let doc = self.gen_doc();
            let enum_code = self.gen_enum(variants);
            quote! {
                #doc
                #enum_code
            }
        } else {
            quote! {}
        }
    }

    fn gen_enum(&self, variants: &[Variant]) -> TokenStream {
        self.validate_enum_ids(variants);

        let name = ident(&self.name);

        let variant_defs: Vec<_> = variants
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

        let from_impl = quote! {
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

        let display_impl = quote! {
            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        #(#display_arms),*
                    }
                }
            }
        };

        quote! {
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
            #[repr(u8)]
            pub enum #name {
                #(#variant_defs),*
            }

            #from_impl
            #display_impl
        }
    }

    fn validate_enum_ids(&self, variants: &[Variant]) {
        let mut ids = HashSet::new();

        for variant in variants {
            if !ids.insert(variant.id) {
                panic!(
                    "{}::{} tried to use ID {} but it's already taken! Please take extreme caution to make sure IDs are consistent with old versions",
                    self.name, variant.name, variant.id
                );
            }
        }

        if let (Some(&min), Some(&max)) = (ids.iter().min(), ids.iter().max()) {
            for id in min..=max {
                if !ids.contains(&id) {
                    panic!("ID {} was skipped in {}!", id, self.name);
                }
            }
        }
    }

    fn make_doc_parts(&self) -> Vec<String> {
        let mut doc_parts = Vec::new();

        if !self.desc.is_empty() {
            doc_parts.push(self.desc.to_string());
        }

        if !self.related.is_empty() {
            if !doc_parts.is_empty() {
                doc_parts.push(String::new());
            }
            doc_parts.push("Usually paired with:".to_string());

            for rel in &self.related {
                doc_parts.push(format!(" - [{}] {}", rel.name, rel.reason));
            }
        }

        doc_parts
    }

    fn gen_doc(&self) -> TokenStream {
        let doc_parts = self.make_doc_parts();

        if doc_parts.is_empty() {
            quote! {}
        } else {
            let combined_doc = doc_parts.join("\n");
            quote! { #[doc = #combined_doc] }
        }
    }
}
