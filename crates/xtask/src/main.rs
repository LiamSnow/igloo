use quote::quote;
use std::{env, fs, path::Path};

mod agg;
mod comps;
mod convert;
mod enums;
mod model;
mod types;
use model::*;

const INPUT_FILE: &str = "components.toml";
const LOCK_FILE: &str = "components.lock";
const OUT_FILE: &str = "crates/interface/src/generated.rs";

pub fn main() {
    let task = env::args().nth(1);
    if task.as_deref() != Some("codegen") {
        panic!("Unknown option. Expected `cargo xtask codegen`");
    }

    let workspace_dir = Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();

    let input_file = workspace_dir.join(INPUT_FILE);
    let lock_file = workspace_dir.join(LOCK_FILE);

    let input_contents = fs::read_to_string(&input_file).expect("Failed to read components file");
    let input_toml = toml::from_str(&input_contents).expect("Failed to parse components file");

    let lock_contents = fs::read_to_string(&lock_file).expect("Failed to read components file");
    let lock_toml = toml::from_str(&lock_contents).expect("Failed to parse components file");

    validate(&input_toml, &lock_toml);

    let comps = input_toml.components;

    let comp_enum = comps::gen_comp_enum(&comps);
    let comp_inner = convert::gen_comp_inner(&comps);
    let comp_from_string = convert::gen_comp_from_string(&comps);
    let to_igloo_value = convert::gen_to_igloo_value(&comps);
    let from_igloo_value = convert::gen_from_igloo_value(&comps);

    let aggregator = agg::gen_aggregator(&comps);

    let num_comps = comps.len();
    let comp_type = types::gen_comp_type(&comps);
    let str_funcs = types::gen_str_funcs(&comps);
    let enum_types = enums::gen_enum_types(&comps);
    let comp_igloo_type = types::gen_comp_igloo_type(&comps);
    let enum_comps = comps::gen_enum_comps(&comps);

    let code = quote! {
        // THIS IS GENERATED CODE - DO NOT MODIFY

        use crate::types::*;
        use crate::types::agg::AggregationOp;
        use std::cmp::Ordering;
        use serde::{Serialize, Deserialize};

        /// Total number of Components in Igloo
        /// (length of `Components` enum)
        pub const NUM_COMPONENTS: usize = #num_comps;

        #comp_type

        #enum_types

        #comp_igloo_type

        #enum_comps

        #str_funcs

        #comp_enum

        #comp_inner

        #comp_from_string

        #to_igloo_value

        #from_igloo_value

        #aggregator
    };

    let syntax_tree = match syn::parse2::<syn::File>(code.clone()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to parse generated code: {e}");
            eprintln!("{code}");
            panic!()
        }
    };
    let formatted = prettyplease::unparse(&syntax_tree);

    // save generated
    let out_file = workspace_dir.join(OUT_FILE);
    fs::write(&out_file, formatted).expect("Failed to write out.rs");

    // save lock
    fs::write(&lock_file, input_contents).expect("Failed to write lock");
}

fn validate(input: &ComponentsFile, lock: &ComponentsFile) {
    let input_map = input.make_map(INPUT_FILE);
    let lock_map = lock.make_map(LOCK_FILE);

    for lock_comp in lock_map.values() {
        let Some(inp_comp) = input_map.get(&lock_comp.name) else {
            panic!(
                "Component `{}` was renamed or removed! Cancelling build.",
                lock_comp.name
            );
        };

        use ComponentKind::*;
        if let Enum { variants, .. } = &lock_comp.kind {
            validate_enum(inp_comp, variants);
        } else if lock_comp.kind != inp_comp.kind {
            panic!(
                "Component `{}`'s kind was changed from `{:?}`. Cancelling build.",
                lock_comp.name, lock_comp.kind,
            );
        }
    }
}

fn validate_enum(input: &&Component, lock: &[Variant]) {
    let name = &input.name;

    let ComponentKind::Enum {
        variants: input, ..
    } = &input.kind
    else {
        panic!("Component `{name}`'s kind was changed from `enum`. Cancelling build.",);
    };

    if input.len() < lock.len() {
        panic!("An enum variant was removed Component `{name}`. Cancelling build.",);
    }

    for i in 0..lock.len() {
        if input[i].name != lock[i].name {
            panic!(
                "Variant `{}` was removed or renamed from Component `{name}`. Cancelling build.",
                lock[i].name
            );
        }
    }
}
