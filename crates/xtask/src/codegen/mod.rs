use anyhow::Result;
use std::path::Path;

// mod core;
mod parser;
mod utils;
mod wit;

const WIT_DIR: &str = "wit";
const WIT_PACKAGE: &str = "igloo:lib@0.1.0";
const COMPS_INPUT: &str = "components.txt";
const COMPS_OUTPUT: &str = "components.wit";
const OUT_DIR: &str = "crates/interface/src/generated";

const SHARED_WORLD: &str = "shared";
const GATED_WORLDS: [&str; 3] = ["core", "extension-sp", "extension-mp"];

pub fn run() -> Result<()> {
    let workspace_dir = Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf();

    let wit_dir = workspace_dir.join(WIT_DIR);
    let out_dir = workspace_dir.join(OUT_DIR);

    // parse file
    let comps = parser::parse(&wit_dir.join(COMPS_INPUT))?;

    // transpile to wit
    wit::make_comps_wit_file(&wit_dir.join(COMPS_OUTPUT), &comps)?;

    // compile all wit
    wit::compile_wit(&wit_dir, &out_dir)?;

    // gen supporting rust core code
    // gen_rust_comp(&out_dir, &comps)?;

    Ok(())
}
