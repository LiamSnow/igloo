use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;

mod model;
mod rust;

pub use model::*;

pub fn run() {
    println!("cargo:rerun-if-changed=components.toml");

    // read toml file
    let man_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml_path = Path::new(&man_dir).join("components.toml");
    let toml_content = fs::read_to_string(toml_path).expect("Failed to read components.toml");
    let config: ComponentsConfig =
        toml::from_str(&toml_content).expect("Failed to parse components.toml");
    let mut comps = config.components;

    // add more components based off gen_* flags
    add_gen_comps(&mut comps);

    // make sure no IDs conflict or are skipped
    validate_component_ids(&comps);

    // sort so Borsh actually uses the ID
    // (because its using the ordinal)
    comps.sort_by_key(|comp| comp.id);

    rust::gen_code(&comps);
}

fn validate_component_ids(comps: &[Component]) {
    let mut ids = HashSet::new();

    for comp in comps {
        if !ids.insert(comp.id) {
            panic!(
                "Component {} tried to use ID {} but it's already taken! Please take extreme caution to make sure IDs are consistent with old versions",
                comp.name, comp.id
            );
        }
    }

    let min = *ids.iter().min().unwrap();
    let max = *ids.iter().max().unwrap();
    if min != 0 {
        panic!("Component ID 0+ was skipped!");
    }

    for id in min..=max {
        if !ids.contains(&id) {
            panic!("Component ID {} was skipped!", id);
        }
    }
}

fn add_gen_comps(comps: &mut Vec<Component>) {
    let mut new_comps = Vec::new();

    for comp in comps.iter_mut() {
        if let ComponentKind::Single {
            field,
            gen_bound_types,
            gen_inner_list_type,
            gen_string_bound_types,
            ..
        } = &comp.kind
        {
            if let Some(ids) = gen_bound_types {
                add_gen_bound_types(&mut new_comps, comp, field, ids);
            }

            if let Some(id) = gen_inner_list_type {
                add_gen_inner_list_type(&mut new_comps, comp, field, *id);
            }

            if let Some(ids) = gen_string_bound_types {
                add_gen_string_bound_types(&mut new_comps, comp, field, ids);
            }
        }

        if let Some(id) = comp.gen_supported_type {
            add_gen_supported_type(&mut new_comps, comp, id);
        }

        if let Some(id) = comp.gen_list_type {
            add_gen_list_type(&mut new_comps, comp, id);
        }
    }

    comps.append(&mut new_comps);
}

fn add_gen_bound_types(
    new_comps: &mut Vec<Component>,
    comp: &Component,
    field: &str,
    ids: &[u16; 3],
) {
    for (i, suffix) in ["Min", "Max", "Step"].iter().enumerate() {
        let new_name = format!("{}{}", comp.name, suffix);
        let reason = match *suffix {
            "Min" => "sets a lower bound on",
            "Max" => "sets an upper bound on",
            "Step" => "sets a step requirement on",
            _ => unreachable!(),
        }
        .to_string();

        new_comps.push(Component {
            name: new_name,
            id: ids[i],
            desc: "Marks a bound. Only enforced by dashboard components that use it.".to_string(),
            gen_supported_type: None,
            gen_list_type: None,
            kind: ComponentKind::single(field.to_string()),
            related: vec![Related {
                name: comp.name.clone(),
                reason,
            }],
        });
    }
}

fn add_gen_inner_list_type(new_comps: &mut Vec<Component>, comp: &Component, field: &str, id: u16) {
    new_comps.push(Component {
        name: format!("{}List", comp.name),
        id,
        desc: format!("a variable-length list of {}", field),
        related: Vec::new(),
        gen_supported_type: None,
        gen_list_type: None,
        kind: ComponentKind::single(format!("Vec<{}>", field)),
    });
}

fn add_gen_string_bound_types(
    new_comps: &mut Vec<Component>,
    comp: &Component,
    field: &str,
    ids: &[u16; 3],
) {
    if field != "String" {
        panic!("Cannot implement String bound types for a non-String {field}")
    }

    for (i, suffix) in ["MaxLength", "MinLength", "Pattern"].iter().enumerate() {
        let new_name = format!("{}{}", comp.name, suffix);
        let (reason, field_type) = match *suffix {
            "MaxLength" => ("sets a max length bound on".to_string(), "u32".to_string()),
            "MinLength" => ("sets a min length bound on".to_string(), "u32".to_string()),
            "Pattern" => ("set a regex pattern for".to_string(), "String".to_string()),
            _ => unreachable!(),
        };

        new_comps.push(Component {
            name: new_name,
            id: ids[i],
            desc: "Marks a requirement. Only enforced by dashbaord components that use it."
                .to_string(),
            gen_supported_type: None,
            gen_list_type: None,
            kind: ComponentKind::single(field_type),
            related: vec![Related {
                name: comp.name.clone(),
                reason,
            }],
        });
    }
}

fn add_gen_supported_type(new_comps: &mut Vec<Component>, comp: &mut Component, id: u16) {
    new_comps.push(Component {
        name: format!("Supported{}s", comp.name),
        id,
        desc: format!("specifies what {}s are supported by this entity", comp.name),
        related: Vec::new(),
        gen_supported_type: None,
        gen_list_type: None,
        kind: ComponentKind::single(format!("Vec<{}>", comp.name)),
    });

    comp.related.push(Related {
        name: format!("Supported{}s", comp.name),
        reason: "specifies what is supported by the entity".to_string(),
    });
}

fn add_gen_list_type(new_comps: &mut Vec<Component>, comp: &Component, id: u16) {
    new_comps.push(Component {
        name: format!("{}List", comp.name),
        id,
        desc: format!("A list of {}", comp.name),
        related: Vec::new(),
        gen_supported_type: None,
        gen_list_type: None,
        kind: ComponentKind::single(format!("Vec<{}>", comp.name)),
    });
}

impl ComponentKind {
    fn single(field: String) -> Self {
        ComponentKind::Single {
            field,
            gen_bound_types: None,
            gen_inner_list_type: None,
            gen_string_bound_types: None,
        }
    }
}
