use super::model::*;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::{collections::HashSet, env, fs, path::PathBuf};
use syn::Ident;

pub fn generate(cmds: &[Command], comps: &[Component]) {
    let max_id = comps.iter().map(|comp| comp.id).max().unwrap();
    let cmd_ids = gen_cmd_ids(cmds, comps);
    let comp_type = gen_comp_type(comps);
    let helper_funcs = gen_helper_funcs(cmds, comps);
    let str_funcs = gen_str_funcs(comps);
    let enum_types = gen_enum_types(comps);
    let comp_igloo_type = gen_comp_igloo_type(comps);
    let comps = gen_comps(comps);
    let cmds = gen_cmd_payloads(cmds);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY

        use borsh::{BorshSerialize, BorshDeserialize};
        use crate::types::*;
        use crate::compound::*;
        #[cfg(feature = "floe")]
        use tokio::io::AsyncWriteExt;
        #[cfg(feature = "penguin")]
        use serde::{Deserialize, Serialize};

        pub const MAX_SUPPORTED_COMPONENT: u16 = #max_id;

        // 0-31 reserved for Igloo <-> Floe
        // 31-63 reserved for Custom Floe Commands (specified in Floe.toml)
        // 64+ reserved for Component commands
        #cmd_ids

        #comp_type

        #enum_types

        #comp_igloo_type

        #comps

        #cmds

        #helper_funcs

        #str_funcs
    };

    // reconstruct, format, and save
    let syntax_tree = syn::parse2::<syn::File>(code).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // save to target/ dir
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("out.rs");
    fs::write(&out_path, formatted).expect("Failed to write out.rs");
}

fn gen_str_funcs(comps: &[Component]) -> TokenStream {
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

fn gen_comp_igloo_type(comps: &[Component]) -> TokenStream {
    let arms: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);

            match &comp.kind {
                ComponentKind::Single { kind } => {
                    let igloo_type = match kind {
                        IglooType::Integer => quote! { IglooType::Integer },
                        IglooType::Real => quote! { IglooType::Real },
                        IglooType::Text => quote! { IglooType::Text },
                        IglooType::Boolean => quote! { IglooType::Boolean },
                        IglooType::Color => quote! { IglooType::Color },
                        IglooType::Date => quote! { IglooType::Date },
                        IglooType::Time => quote! { IglooType::Time },
                        IglooType::IntegerList => quote! { IglooType::IntegerList },
                        IglooType::RealList => quote! { IglooType::RealList },
                        IglooType::TextList => quote! { IglooType::TextList },
                        IglooType::BooleanList => quote! { IglooType::BooleanList },
                        IglooType::ColorList => quote! { IglooType::ColorList },
                        IglooType::DateList => quote! { IglooType::DateList },
                        IglooType::TimeList => quote! { IglooType::TimeList },
                    };
                    quote! {
                        ComponentType::#name => Some(#igloo_type)
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

fn gen_helper_funcs(cmds: &[Command], comps: &[Component]) -> TokenStream {
    let cmds: Vec<_> = cmds
        .iter()
        .map(|cmd| {
            gen_helper_func(
                &cmd.name,
                &upper_camel_to_screaming_snake(&cmd.name).to_lowercase(),
                &upper_camel_to_screaming_snake(&cmd.name),
                !cmd.fields.is_empty(),
            )
        })
        .collect();
    let comps: Vec<_> = comps
        .iter()
        .map(|comp| {
            gen_helper_func(
                &comp.name,
                &upper_camel_to_snake(&comp.name),
                &comp_name_to_cmd_name(&comp.name),
                !comp.is_marker(),
            )
        })
        .collect();
    quote! {
        #[cfg(feature = "floe")]
        impl<W: AsyncWriteExt + Unpin> crate::floe::FloeWriter<W> {
            #(#cmds)*
            #(#comps)*
        }
    }
}

fn gen_helper_func(name: &str, func_name: &str, cmd_name: &str, has_payload: bool) -> TokenStream {
    let func_name = ident(func_name);
    let cmd_name = ident(cmd_name);
    let name = ident(name);

    if has_payload {
        quote! {
            pub async fn #func_name(
                &mut self,
                payload: &#name,
            ) -> Result<(), std::io::Error> {
                self.write_with_payload(#cmd_name, payload).await
            }
        }
    } else {
        quote! {
            pub async fn #func_name(&mut self) -> Result<(), std::io::Error> {
                self.write_no_payload(#cmd_name).await
            }
        }
    }
}

fn gen_comp_type(comps: &[Component]) -> TokenStream {
    let variants: Vec<_> = comps
        .iter()
        .map(|comp| {
            let name = ident(&comp.name);
            let id = comp.id;
            quote! { #name = #id }
        })
        .collect();

    quote! {
        #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, BorshSerialize, BorshDeserialize)]
        #[repr(u16)]
        #[borsh(use_discriminant=false)]
        pub enum ComponentType {
            #(#variants),*
        }
    }
}

fn gen_cmd_payloads(cmds: &[Command]) -> TokenStream {
    let loads: Vec<_> = cmds
        .iter()
        .map(|cmd| {
            let load = cmd.gen_payload();
            quote! { #load }
        })
        .collect();

    quote! {
        #(#loads)*
    }
}

impl Command {
    fn gen_payload(&self) -> TokenStream {
        if self.fields.is_empty() {
            return quote! {};
        }

        let name = ident(&self.name);
        let desc = &self.desc;

        let field_defs: Vec<_> = self
            .fields
            .iter()
            .map(|field| {
                let field_name = ident(&field.name);
                let field_type = field
                    .r#type
                    .parse::<TokenStream>()
                    .unwrap_or_else(|_| panic!("Failed to parse type: {}", field.r#type));
                let desc = &field.desc;

                if desc.is_empty() {
                    quote! {
                        pub #field_name: #field_type
                    }
                } else {
                    quote! {
                        #[doc = #desc]
                        pub #field_name: #field_type
                    }
                }
            })
            .collect();

        quote! {
            #[doc = #desc]
            #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
            pub struct #name {
                #(#field_defs),*
            }
        }
    }
}

pub fn gen_cmd_ids(cmds: &[Command], comps: &[Component]) -> TokenStream {
    let cmds: Vec<_> = cmds.iter().map(|cmd| cmd.gen_id_const()).collect();
    let comps: Vec<_> = comps.iter().map(|comp| comp.gen_id_const()).collect();

    // TODO FIXME place into module

    quote! {
        #(#cmds)*
        #(#comps)*
    }
}

impl Command {
    fn gen_id_const(&self) -> TokenStream {
        let scream = ident(&upper_camel_to_screaming_snake(&self.name));
        let id = self.id;

        let name_part = if !self.fields.is_empty() {
            format!("[{}]", self.name)
        } else {
            self.name.clone()
        };
        let intro_doc = format!("Command ID for writing a {}", name_part);
        let desc = &self.desc;

        quote! {
            #[doc = #intro_doc]
            #[doc = #desc]
            pub const #scream: u16 = #id;
        }
    }
}

impl Component {
    fn gen_id_const(&self) -> TokenStream {
        let cmd_name = ident(&comp_name_to_cmd_name(&self.name));
        let id = self.id + 64;

        let name_part = if !self.is_marker() {
            format!("[{}]", self.name)
        } else {
            self.name.clone()
        };
        let intro_doc = format!("Command ID for writing a {}", name_part);
        let component_doc = self.gen_doc();

        quote! {
            #[doc = #intro_doc]
            #component_doc
            pub const #cmd_name: u16 = #id;
        }
    }
}

fn gen_comps(comps: &[Component]) -> TokenStream {
    let comps: Vec<_> = comps.iter().map(|comp| comp.gen_code()).collect();

    // TODO FIXME place into module

    quote! {
        #(#comps)*
    }
}

impl Component {
    fn gen_code(&self) -> TokenStream {
        let doc = self.gen_doc();
        let comp_code = match &self.kind {
            ComponentKind::Single { kind, .. } => self.gen_single(kind),
            ComponentKind::Enum { variants, .. } => self.gen_enum(variants),
            ComponentKind::Marker { .. } => quote! {}, // no data
        };
        quote! {
            #doc
            #comp_code
        }
    }

    pub fn is_marker(&self) -> bool {
        matches!(self.kind, ComponentKind::Marker { .. })
    }

    fn gen_single(&self, r#type: &IglooType) -> TokenStream {
        let name = ident(&self.name);
        let field_type = format_ident!("Igloo{type:?}");
        quote! {
            pub type #name = #field_type;
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
            #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
            #[cfg_attr(feature = "penguin", derive(Serialize, Deserialize))]
            #[borsh(use_discriminant = true)]
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

fn gen_enum_types(comps: &[Component]) -> TokenStream {
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
            if let ComponentKind::Enum { variants, .. } = &comp.kind {
                let first_variant = ident(&variants[0].name);
                quote! {
                    IglooEnumType::#name => IglooEnumValue::#name(#name::#first_variant)
                }
            } else {
                unreachable!()
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
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
        #[cfg_attr(feature = "penguin", derive(Serialize, Deserialize))]
        pub enum IglooEnumType {
            #(#type_variants),*
        }

        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        #[cfg_attr(feature = "penguin", derive(Serialize, Deserialize))]
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

/// ex. maps `Integer` -> `SET_INTEGER`
pub fn comp_name_to_cmd_name(comp_name: &str) -> String {
    format!("WRITE_{}", upper_camel_to_screaming_snake(comp_name))
}

pub fn upper_camel_to_screaming_snake(s: &str) -> String {
    let mut res = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            res.push('_');
        }
        res.push(c.to_ascii_uppercase());
    }
    res
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
