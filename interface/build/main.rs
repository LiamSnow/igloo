use std::{collections::HashSet, env, path::Path};

mod deriver;
mod model;
mod rust;
mod server;
use model::*;

const COMPONENTS_FILE: &str = "components.toml";
const PROTOCOL_FILE: &str = "protocol.toml";

pub fn main() {
    println!("cargo:rerun-if-changed={COMPONENTS_FILE}");
    println!("cargo:rerun-if-changed={PROTOCOL_FILE}");

    // read toml files
    let man_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let cmds = ProtocolConfig::read(Path::new(&man_dir).join(PROTOCOL_FILE)).commands;
    let mut comps = ComponentsConfig::read(Path::new(&man_dir).join(COMPONENTS_FILE)).components;

    deriver::add_derived_comps(&mut comps);

    validate_component_ids(&comps);
    validate_command_ids(&cmds);

    rust::generate(&cmds, &comps);
    server::generate(&cmds, &comps);
}

/// makes sure no IDs conflict or are skipped
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

/// makes sure IDs are in reserved zone (0-31)
fn validate_command_ids(cmds: &[Command]) {
    for cmd in cmds {
        if cmd.id > 31 {
            panic!(
                "Command {} has invalid ID {}. Must be 0-31",
                cmd.name, cmd.id
            );
        }
    }
}
