use std::env;
use std::fs;
use std::path::Path;

mod model;
mod rust;

pub use model::*;

pub fn run() {
    println!("cargo:rerun-if-changed=protocol.toml");

    // read toml file
    let man_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml_path = Path::new(&man_dir).join("protocol.toml");
    let toml_content = fs::read_to_string(toml_path).expect("Failed to read protocol.toml");
    let config: ProtocolConfig =
        toml::from_str(&toml_content).expect("Failed to parse protocol.toml");

    rust::gen_code(&config);
}
