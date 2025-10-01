extern crate prost_build;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

fn main() {
    prost_build::compile_protos(&["src/api.proto"], &["src/"]).unwrap();

    // generate MessageType enum in model.rs
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let output_path = PathBuf::from(out_dir).join("model.rs");

    generate_message_type_enum("src/api.proto", &output_path.to_string_lossy())
        .expect("Failed to generate model.rs");
}

fn generate_message_type_enum(proto_path: &str, output_path: &str) -> std::io::Result<()> {
    let messages = parse_proto_messages(proto_path)?;
    write_enum_file(output_path, messages)?;

    println!("cargo:rerun-if-changed=src/api.proto");

    Ok(())
}

fn parse_proto_messages(proto_path: &str) -> std::io::Result<Vec<(String, u16)>> {
    let file = File::open(proto_path)?;
    let reader = BufReader::new(file);

    let mut current_message: Option<String> = None;
    let mut brace_depth = 0;
    let mut messages: Vec<(String, u16)> = Vec::new();

    for line in reader.lines() {
        let line = line?;

        if should_skip_line(&line) {
            continue;
        }

        if let Some(name) = extract_message_name(&line) {
            current_message = Some(name);
            brace_depth = 1;
            continue;
        }

        // not inside message -> skip
        let message_name = match current_message.as_ref() {
            Some(name) => name,
            None => continue,
        };

        update_brace_depth(&line, &mut brace_depth);
        if brace_depth == 0 {
            current_message = None;
            continue;
        }

        if let Some(id) = extract_option_id(&line) {
            messages.push((message_name.clone(), id));
        }
    }

    messages.sort_by_key(|&(_, id)| id);
    Ok(messages)
}

fn should_skip_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.is_empty() || trimmed.starts_with("//")
}

/// ex. `message HelloRequest {`
fn extract_message_name(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if !trimmed.starts_with("message ") || !trimmed.contains('{') {
        return None;
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    Some(parts[1].trim_end_matches('{').to_string())
}

/// ex. `option (id) = 1;`
fn extract_option_id(line: &str) -> Option<u16> {
    let trimmed = line.trim();

    if !trimmed.starts_with("option (id)") && !trimmed.starts_with("option(id)") {
        return None;
    }

    let eq_pos = trimmed.find('=')?;
    let after_eq = &trimmed[eq_pos + 1..];

    let id_str = after_eq.trim().trim_end_matches(';').trim();
    id_str.parse::<u16>().ok()
}

fn update_brace_depth(line: &str, depth: &mut i32) {
    for ch in line.chars() {
        match ch {
            '{' => *depth += 1,
            '}' => *depth -= 1,
            _ => {}
        }
    }
}

fn write_enum_file(output_path: &str, messages: Vec<(String, u16)>) -> std::io::Result<()> {
    let mut content = String::new();

    content.push_str("use strum_macros::{Display, FromRepr};\n");
    content.push('\n');
    content.push_str("#[derive(FromRepr, Display, Debug, PartialEq, Clone)]\n");
    content.push_str("#[repr(u16)]\n");
    content.push_str("pub enum MessageType {\n");

    for (name, id) in &messages {
        content.push_str(&format!("    {} = {},\n", name, id));
    }

    content.push_str("}\n");

    let mut output = File::create(output_path)?;
    output.write_all(content.as_bytes())?;

    Ok(())
}
