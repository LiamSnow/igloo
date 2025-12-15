use miette::{IntoDiagnostic, Result};
use std::env;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename.igd>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = std::fs::read_to_string(filename).into_diagnostic()?;

    let arena = bumpalo::Bump::new();

    match igd::parser::parse(&source, &arena) {
        Ok(ast) => {
            println!("Parsed successfully!");
            println!("{:#?}", ast);
            // ... continue with IR compilation
        }
        Err(e) => {
            // Miette automatically formats this beautifully!
            return Err(e.into());
        }
    }

    Ok(())
}
