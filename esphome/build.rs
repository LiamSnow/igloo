extern crate prost_build;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use syn::Ident;

fn main() {
    println!("cargo:rerun-if-changed=src/api.proto");

    prost_build::compile_protos(&["src/api.proto"], &["src/"]).unwrap();

    let msgs = parse_proto_messages("src/api.proto");
    let entities = extract_entity_types(&msgs);
    let states = extract_state_responses(&msgs);

    let msg_enum = gen_message_type_enum(&msgs);
    let entity_enum = gen_entity_type_enum(&entities);
    let process_fn = gen_process_state_update(&states);
    let register_fn = gen_register_entities(&entities);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY

        use strum_macros::{Display, FromRepr};
        use crate::api;
        use prost::Message;
        use crate::entity::{EntityRegister};
        use crate::device::{Device, DeviceError};
        use bytes::BytesMut;
        use igloo_interface::FloeWriterDefault;
        use crate::connection::base::Connectionable;

        #msg_enum

        #entity_enum

        #process_fn

        #register_fn
    };

    let syntax = syn::parse2::<syn::File>(code).unwrap();
    let formatted = prettyplease::unparse(&syntax);

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("model.rs");
    fs::write(&out_path, formatted).unwrap();
}

fn parse_proto_messages(path: &str) -> Vec<(String, u16)> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut current: Option<String> = None;
    let mut depth = 0;
    let mut msgs = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if let Some(name) = extract_message_name(&line) {
            current = Some(name);
            depth = 1;
            continue;
        }

        let msg_name = match current.as_ref() {
            Some(n) => n,
            None => continue,
        };

        update_depth(&line, &mut depth);
        if depth == 0 {
            current = None;
            continue;
        }

        if let Some(id) = extract_option_id(&line) {
            msgs.push((msg_name.clone(), id));
        }
    }

    msgs.sort_by_key(|&(_, id)| id);
    msgs
}

fn extract_message_name(line: &str) -> Option<String> {
    let t = line.trim();
    if !t.starts_with("message ") || !t.contains('{') {
        return None;
    }
    let parts: Vec<&str> = t.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    Some(parts[1].trim_end_matches('{').to_string())
}

fn extract_option_id(line: &str) -> Option<u16> {
    let t = line.trim();
    if !t.starts_with("option (id)") && !t.starts_with("option(id)") {
        return None;
    }
    let eq = t.find('=')?;
    let after = &t[eq + 1..];
    let id_str = after.trim().trim_end_matches(';').trim();
    id_str.parse::<u16>().ok()
}

fn update_depth(line: &str, depth: &mut i32) {
    for ch in line.chars() {
        match ch {
            '{' => *depth += 1,
            '}' => *depth -= 1,
            _ => {}
        }
    }
}

fn extract_entity_types(msgs: &[(String, u16)]) -> Vec<String> {
    msgs.iter()
        .filter_map(|(name, _)| {
            if !name.starts_with("ListEntities")
                || !name.ends_with("Response")
                || name == "ListEntitiesDoneResponse"
                || name == "ListEntitiesServicesResponse"
            {
                return None;
            }
            let start = "ListEntities".len();
            let end = name.len() - "Response".len();
            if start >= end {
                return None;
            }
            Some(name[start..end].to_string())
        })
        .collect()
}

fn extract_state_responses(msgs: &[(String, u16)]) -> Vec<String> {
    msgs.iter()
        .filter_map(|(name, _)| {
            if name.ends_with("StateResponse") && !name.contains("HomeAssistant") {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect()
}

fn gen_message_type_enum(msgs: &[(String, u16)]) -> TokenStream {
    let variants = msgs.iter().map(|(name, id)| {
        let ident = Ident::new(name, Span::call_site());
        quote! { #ident = #id }
    });

    quote! {
        #[derive(FromRepr, Display, Debug, PartialEq, Clone)]
        #[repr(u16)]
        pub enum MessageType {
            #(#variants,)*
        }
    }
}

fn gen_entity_type_enum(entities: &[String]) -> TokenStream {
    let variants = entities.iter().map(|name| {
        let ident = Ident::new(name, Span::call_site());
        quote! { #ident }
    });

    quote! {
        #[derive(Clone, Debug)]
        pub enum EntityType {
            #(#variants,)*
        }
    }
}

fn gen_process_state_update(states: &[String]) -> TokenStream {
    let arms = states.iter().map(|name| {
        let msg_type = Ident::new(name, Span::call_site());
        let api_type = Ident::new(name, Span::call_site());
        quote! {
            MessageType::#msg_type => {
                self.apply_entity_update(api::#api_type::decode(msg)?).await?;
            }
        }
    });

    quote! {
        impl Device {
            pub async fn process_state_update(
                &mut self,
                msg_type: MessageType,
                msg: BytesMut,
            ) -> Result<(), DeviceError> {
                match msg_type {
                    MessageType::DisconnectRequest
                    | MessageType::PingRequest
                    | MessageType::PingResponse
                    | MessageType::GetTimeRequest
                    | MessageType::SubscribeLogsResponse => {
                        unreachable!()
                    }
                    #(#arms)*
                    _ => {}
                }
                Ok(())
            }
        }
    }
}

fn gen_register_entities(entities: &[String]) -> TokenStream {
    let arms = entities.iter().map(|entity| {
        let msg = Ident::new(
            &format!("ListEntities{}Response", entity),
            Span::call_site(),
        );
        let api = Ident::new(
            &format!("ListEntities{}Response", entity),
            Span::call_site(),
        );
        quote! {
            MessageType::#msg => {
                let msg = api::#api::decode(msg)?;
                msg.register(self, writer).await?;
            }
        }
    });

    quote! {
        impl Device {
            pub async fn register_entities(
                &mut self,
                writer: &mut FloeWriterDefault,
                device_idx: u16,
            ) -> Result<(), DeviceError> {
                self.send_msg(
                    MessageType::ListEntitiesRequest,
                    &api::ListEntitiesRequest {},
                ).await?;

                self.device_idx = Some(device_idx);

                loop {
                    let (msg_type, msg) = self.connection.recv_msg().await?;

                    match msg_type {
                        MessageType::ListEntitiesServicesResponse => {
                            continue;
                        }
                        MessageType::ListEntitiesDoneResponse => break,
                        #(#arms)*
                        _ => continue,
                    }

                    writer.deselect_entity().await?;
                }

                Ok(())
            }
        }
    }
}
