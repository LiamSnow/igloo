use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::env;
use std::fs;
use std::path::PathBuf;

use super::*;

pub fn gen_code(config: &ProtocolConfig) {
    let version = config.version;
    let cmd_payloads = gen_cmd_payloads(&config.commands);
    let floe_variants = gen_cmd_variants(&config.commands.floe);
    let igloo_variants = gen_cmd_variants(&config.commands.igloo);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY
        // Generated from protocol.toml by build.rs

        use borsh::{BorshDeserialize, BorshSerialize};
        use crate::Component;

        pub const PROTOCOL_VERSION: u8 = #version;

        /// MISO Floe sending command -> Igloo
        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        #[borsh(use_discriminant = true)]
        #[repr(u8)]
        pub enum FloeCommand {
            #floe_variants
        }

        /// MOSI Igloo sending command -> Floe
        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        #[borsh(use_discriminant = true)]
        #[repr(u8)]
        pub enum IglooCommand {
            #igloo_variants
        }

        #cmd_payloads
    };

    // reconstruct, format, and save
    let syntax_tree = syn::parse2::<syn::File>(code).expect("Failed to parse generated code");
    let formatted = prettyplease::unparse(&syntax_tree);

    // save to target/ dir
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("protocol.rs");
    fs::write(&out_path, formatted).expect("Failed to write protocol.rs");
}

fn gen_cmd_variants(cmds: &[Command]) -> TokenStream {
    let variants: Vec<_> = cmds
        .iter()
        .map(|cmd| {
            let name = ident(&cmd.name);
            let payload = ident(&format!("{}Payload", cmd.name));
            let desc = &cmd.desc;
            let opcode = cmd.opcode;
            quote! {
                #[doc = #desc]
                #name(#payload) = #opcode
            }
        })
        .collect();

    quote! {
        #(#variants),*
    }
}

fn gen_cmd_payloads(cmds: &Commands) -> TokenStream {
    let defs: Vec<_> = cmds
        .floe
        .iter()
        .chain(cmds.igloo.iter())
        .map(|cmd| {
            let cmd_payload = gen_cmd_payload(cmd);
            quote! { #cmd_payload }
        })
        .collect();

    quote! {
        #(#defs)*
    }
}

/// returns
fn gen_cmd_payload(cmd: &Command) -> TokenStream {
    let name = ident(&format!("{}Payload", cmd.name));

    let field_defs: Vec<_> = cmd
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

    let desc = &cmd.desc;

    quote! {
        #[doc = #desc]
        #[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize)]
        pub struct #name {
            #(#field_defs),*
        }
    }
}

fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
