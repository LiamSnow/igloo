use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=components.rs");

    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let components_path = Path::new(&out_dir).join("components.rs");
    let output_path = Path::new(&out_dir).join("src/tree.rs");

    let content = fs::read_to_string(&components_path).expect("Failed to read components.rs");

    let mut output = String::new();
    let mut component_names = Vec::new();

    output.push_str("// THIS IS GENERATED CODE - DO NOT MODIFY\n");
    output.push_str("// Generated from components.rs by build.rs\n\n");

    output.push_str("use serde::{Deserialize, Serialize};\n");
    output.push_str("use crate::Entity;\n\n");

    for line in content.lines() {
        if line.starts_with("struct ") || line.starts_with("enum ") {
            // add `pub` and `#[derive(..)]` to structs and enums
            let name = extract_component_name(line);
            component_names.push(name);

            output.push_str("#[derive(Debug, Serialize, Deserialize, Clone)]\n");

            let public_line = line
                .replace("struct ", "pub struct ")
                .replace("enum ", "pub enum ");
            output.push_str(&public_line);
            output.push('\n');
        } else {
            // keep everything else
            output.push_str(line);
            output.push('\n');
        }
    }

    // ComponentType
    output.push_str("#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]\n");
    output.push_str("#[repr(u16)]\n");
    output.push_str("pub enum ComponentType {\n");
    for name in &component_names {
        output.push_str(&format!("\t{name},\n"));
    }
    output.push_str("}\n\n");

    // Component
    output.push_str("#[derive(Debug, Serialize, Deserialize, Clone)]\n");
    output.push_str("pub enum Component {\n");
    for name in &component_names {
        output.push_str(&format!("\t{name}({name}),\n"));
    }
    output.push_str("}\n\n");

    // Component::get_type()
    output.push_str("impl Component {\n");
    output.push_str("\tpub fn get_type(&self) -> ComponentType {\n");
    output.push_str("\t\tmatch self {\n");
    for name in &component_names {
        output.push_str(&format!(
            "\t\t\tComponent::{name}(_) => ComponentType::{name},\n",
        ));
    }
    output.push_str("\t\t}\n");
    output.push_str("\t}\n");
    output.push_str("}\n\n");

    // Entity methods
    output.push_str("impl Entity {\n");
    for name in &component_names {
        let snake_name = to_snake_case(name);

        // imut getter
        output.push_str(&format!(
            "\tpub fn {snake_name}(&self) -> Option<&{name}> {{\n",
        ));
        output.push_str(&format!(
            "\t\tmatch self.0.get(&ComponentType::{name}) {{\n",
        ));
        output.push_str(&format!(
            "\t\t\tSome(Component::{name}(val)) => Some(val),\n",
        ));
        output.push_str("\t\t\tSome(_) => panic!(\"Entity Type/Value Mismatch!\"),\n");
        output.push_str("\t\t\tNone => None,\n");
        output.push_str("\t\t}\n");
        output.push_str("\t}\n\n");

        // mut getter
        output.push_str(&format!(
            "\tpub fn {snake_name}_mut(&mut self) -> Option<&mut {name}> {{\n",
        ));
        output.push_str(&format!(
            "\t\tmatch self.0.get_mut(&ComponentType::{name}) {{\n",
        ));
        output.push_str(&format!(
            "\t\t\tSome(Component::{name}(val)) => Some(val),\n",
        ));
        output.push_str("\t\t\tSome(_) => panic!(\"Entity Type/Value Mismatch!\"),\n");
        output.push_str("\t\t\tNone => None,\n");
        output.push_str("\t\t}\n");
        output.push_str("\t}\n\n");

        // setter
        output.push_str(&format!(
            "    pub fn set_{snake_name}(&mut self, val: {name}) {{\n",
        ));
        output.push_str(&format!(
            "        self.0.insert(ComponentType::{name}, Component::{name}(val));\n",
        ));
        output.push_str("    }\n");

        if name != component_names.last().unwrap() {
            output.push('\n');
        }

        // has
        output.push_str(&format!("\tpub fn has_{snake_name}(&self) -> bool {{\n",));
        output.push_str(&format!("\t\tself.{snake_name}().is_some()\n",));
        output.push_str("\t}\n\n");
    }
    output.push_str("}\n");

    fs::write(&output_path, output).expect("Failed to write tree.rs");

    println!(
        "Generated {} components in src/tree.rs",
        component_names.len()
    );
}

fn extract_component_name(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1]
            .chars()
            .take_while(|c| c.is_alphanumeric())
            .collect()
    } else {
        panic!("Invalid struct/enum definition: {line}");
    }
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut prev_upper = false;

    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !prev_upper {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_upper = true;
        } else {
            result.push(ch);
            prev_upper = false;
        }
    }

    result
}
