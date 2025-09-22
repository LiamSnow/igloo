use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

// I hate macros so here this is my hacky (but fast) code generation!

fn main() {
    println!("cargo:rerun-if-changed=components.rs");

    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let components_path = Path::new(&out_dir).join("components.rs");
    let output_path = Path::new(&out_dir).join("src/components.rs");

    let content = fs::read_to_string(&components_path).expect("Failed to read components.rs");

    let mut output = String::new();
    let mut component_names = Vec::new();

    output.push_str("// THIS IS GENERATED CODE - DO NOT MODIFY\n");
    output.push_str("// Generated from components.rs by build.rs\n\n");

    output.push_str("use serde::{Deserialize, Serialize};\n");
    output.push_str("use crate::{Entity, Averageable};\n");
    output.push_str("use std::ops::{Add, Sub};\n\n");

    let lines: Vec<&str> = content.lines().collect();

    // FIXME this hurts to read
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];

        let is_struct = line.starts_with("struct ");
        let is_enum = line.starts_with("enum ");
        if is_struct || is_enum {
            output.push_str("#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]\n");
            output.push_str("pub ");

            // read entire struct|enum
            let component_name = extract_component_name(line);

            // IE `struct NAME(field);`
            let is_oneline = line.contains(";");
            if is_oneline {
                output.push_str(line);
                output.push('\n');
                if is_struct && let Some(lines) = implement_easy_averagable(line, &component_name) {
                    output.push_str(&lines);
                }
            } else {
                let mut body = line.to_string();
                body.push('\n');
                i += 1;
                if !line.contains("{") {
                    panic!("struct|enum has invalid syntax: {line}");
                }

                let mut stack_counter = 1; // we have 1 { to start
                while stack_counter != 0 && i < lines.len() {
                    let line = lines[i];
                    if line.contains("{") {
                        stack_counter += 1;
                    } else if line.contains("}") {
                        stack_counter -= 1;
                    }
                    body.push_str(line);
                    body.push('\n');
                    i += 1;
                }

                output.push_str(&body);

                if is_struct && body.contains("IMPLEMENT AVERAGEABLE") {
                    match implement_complex_averagable(body, &component_name) {
                        Ok(lines) => output.push_str(&lines),
                        Err(err) => {
                            panic!("Error implementing averageable for {component_name}: {err}");
                        }
                    }
                }
            }

            component_names.push(component_name);
        } else {
            // everything else
            output.push_str(line);
            output.push('\n');
        }

        i += 1;
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
    output.push_str("#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]\n");
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
            r#"
    pub fn {snake_name}(&self) -> Option<&{name}> {{
        match self.0.get(&ComponentType::{name}) {{
            Some(Component::{name}(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }}
    }}
            "#
        ));

        // mut getter
        output.push_str(&format!(
            r#"
    pub fn {snake_name}_mut(&mut self) -> Option<&mut {name}> {{
        match self.0.get_mut(&ComponentType::{name}) {{
            Some(Component::{name}(val)) => Some(val),
            Some(_) => panic!("Entity Type/Value Mismatch!"),
            None => None,
        }}
    }}
            "#
        ));

        // setter
        output.push_str(&format!(
            r#"
    pub fn set_{snake_name}(&mut self, val: {name}) {{
        self.0.insert(ComponentType::{name}, Component::{name}(val));
    }}
            "#
        ));

        // has
        output.push_str(&format!(
            r#"
    pub fn has_{snake_name}(&self) -> bool {{
        self.{snake_name}().is_some()
    }}
            "#
        ));

        if name != component_names.last().unwrap() {
            output.push('\n');
        }
    }
    output.push_str("}\n");

    fs::write(&output_path, output).expect("Failed to write components.rs");
}

fn extract_component_name(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1]
            .chars()
            .take_while(|c| c.is_alphanumeric())
            .collect()
    } else {
        panic!("struct/enum invalid syntax: {line}");
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

fn get_type_to_sum() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("u8", "u64"),
        ("u16", "u64"),
        ("u32", "u64"),
        ("u64", "u64"),
        ("u128", "u128"),
        ("usize", "usize"),
        ("i8", "i64"),
        ("i16", "i64"),
        ("i32", "i64"),
        ("i64", "i64"),
        ("i128", "i128"),
        ("isize", "isize"),
        ("f32", "f64"),
        ("f64", "f64"),
        ("bool", "u32"),
    ])
}

/// implements averageable for structs which contain ONLY easily averageable fields
/// (ex. u8, i32, etc.)
/// label with "IMPLEMENT AVERAGEABLE" comment inside struct body
fn implement_complex_averagable(body: String, struct_name: &str) -> Result<String, String> {
    let type_to_sum = get_type_to_sum();

    let mut fields = Vec::new();

    for line in body.lines() {
        // pub FIELD: TYPE,
        if !line.contains(":") {
            continue;
        }

        let field_name_start = line
            .find("pub")
            .ok_or(format!("Failed to read field_name start from line: {line}"))?;
        let field_name_end = line
            .find(":")
            .ok_or(format!("Failed to read field_name end from line: {line}"))?;
        let field_name = line[field_name_start + 3..field_name_end].trim();

        let tt_start = line
            .find(":")
            .ok_or(format!("Failed to read type start from line: {line}"))?;
        let tt_end = line
            .find(",")
            .ok_or(format!("Failed to read type end from line: {line}"))?;
        let tt = line[tt_start + 1..tt_end].trim();

        let sum_tt = type_to_sum
            .get(tt)
            .ok_or(format!("{tt} cannot be averaged"))?;

        fields.push((field_name, tt, sum_tt));
    }

    if fields.is_empty() {
        return Err("no fields found".into());
    }

    // make sum struct
    let mut output = "#[derive(Clone, Debug, Default)]\n".to_string();
    output.push_str(&format!("pub struct {struct_name}Sum {{\n"));
    for (field_name, _, sum_tt) in &fields {
        output.push_str(&format!("\tpub {field_name}: {sum_tt},\n"));
    }
    output.push_str("}\n");

    // impl Add and Sub
    for (trait_name, operation) in [("Add", '+'), ("Sub", '-')] {
        output.push_str(&format!("impl {trait_name} for {struct_name}Sum {{\n"));
        output.push_str("\ttype Output = Self;\n");
        output.push_str(&format!(
            "\tfn {}(self, other: Self) -> Self {{\n",
            trait_name.to_lowercase()
        ));
        output.push_str(&format!("\t\t{struct_name}Sum {{\n"));
        for (field_name, _, _) in &fields {
            output.push_str(&format!(
                "\t\t\t{field_name}: self.{field_name} {operation} other.{field_name},\n"
            ));
        }
        output.push_str("\t\t}\n");
        output.push_str("\t}\n");
        output.push_str("}\n");
    }

    // impl Averageable
    output.push_str(&format!("impl Averageable for {struct_name} {{\n"));
    output.push_str(&format!("\ttype Sum = {struct_name}Sum;\n"));

    output.push_str("\tfn to_sum_component(&self) -> Self::Sum {\n");
    output.push_str(&format!("\t\t{struct_name}Sum {{\n"));
    for (field_name, _, sum_tt) in &fields {
        output.push_str(&format!(
            "\t\t\t{field_name}: self.{field_name} as {sum_tt},\n"
        ));
    }
    output.push_str("\t\t}\n");
    output.push_str("\t}\n");

    output.push_str("\tfn from_sum(sum: Self::Sum, len: usize) -> Self {\n");
    output.push_str(&format!("\t\t{struct_name} {{\n"));
    for (field_name, tt, sum_tt) in &fields {
        output.push_str(&format!("\t\t\t{field_name}: "));

        if *tt == "bool" {
            output.push_str(&format!("(sum.{field_name} / len as {sum_tt}) != 0"))
        } else if tt == *sum_tt {
            output.push_str(&format!("sum.{field_name} / len as {sum_tt}"))
        } else {
            output.push_str(&format!("(sum.{field_name} / len as {sum_tt}) as {tt}"))
        }

        output.push_str(",\n");
    }
    output.push_str("\t\t}\n");
    output.push_str("\t}\n");

    output.push_str("}\n\n");

    Ok(output)
}

/// implements averageable for all things that are relatively straight forward
/// only works for single field struct IE
/// `struct NAME(pub TYPE);`
fn implement_easy_averagable(line: &str, struct_name: &str) -> Option<String> {
    let type_to_sum = get_type_to_sum();

    let start = line.find("(pub ")?;
    let end = line.find(");")?;

    let tt = &line[start + 5..end];

    let sum_tt = type_to_sum.get(tt)?;

    let from_sum = if tt == "bool" {
        "(sum / len as Self::Sum) != 0".to_string()
    } else if tt == *sum_tt {
        "sum / len as Self::Sum".to_string()
    } else {
        format!("(sum / len as Self::Sum) as {tt}")
    };

    Some(format!(
        r#"
impl Averageable for {struct_name} {{
    type Sum = {sum_tt};

    fn to_sum_component(&self) -> Self::Sum {{
        self.0 as Self::Sum
    }}

    fn from_sum(sum: Self::Sum, len: usize) -> Self
    where
        Self: Sized,
    {{
        Self({from_sum})
    }}
}}
    "#
    ))
}
