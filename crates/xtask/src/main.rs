use std::env;

use anyhow::Result;

mod codegen;

fn main() -> Result<()> {
    let task = env::args().nth(1);
    if let Some("codegen") = task.as_deref() {
        codegen::run()?
    }
    Ok(())
}
