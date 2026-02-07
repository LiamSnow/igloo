use quote::quote;
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

mod agg;
mod comps;
mod convert;
mod enums;
mod model;
mod types;
use model::*;

const COMPONENTS_FILE: &str = "components.toml";

pub fn main() {
    println!("cargo:rerun-if-changed={COMPONENTS_FILE}");

    let man_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let comps = ComponentsConfig::read(Path::new(&man_dir).join(COMPONENTS_FILE)).components;

    validate_component_ids(&comps);

    let comp_enum = comps::gen_comp_enum(&comps);
    let comp_inner = convert::gen_comp_inner(&comps);
    let comp_from_string = convert::gen_comp_from_string(&comps);
    let to_igloo_value = convert::gen_to_igloo_value(&comps);
    let from_igloo_value = convert::gen_from_igloo_value(&comps);

    let aggregator = agg::gen_aggregator(&comps);

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

    // reconstruct, format, and save
    let syntax_tree = match syn::parse2::<syn::File>(code.clone()) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to parse generated code: {e}");
            eprintln!("{code}");
            panic!()
        }
    };
    let formatted = prettyplease::unparse(&syntax_tree);

    // save to target/ dir
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir).join("out.rs");
    fs::write(&out_path, formatted).expect("Failed to write out.rs");
}

/// makes sure no IDs conflict or are skipped
fn validate_component_ids(comps: &[Component]) {
    let mut names = HashSet::new();

    for comp in comps {
        if !names.insert(&comp.name) {
            panic!("Duplicate components `{}`", comp.name);
        }
    }
}
