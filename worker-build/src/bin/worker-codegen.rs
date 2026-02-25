use anyhow::Context;
use clap::Parser;
use worker_codegen::wit::expand_wit_source;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    version,
    about="Generate RPC service interfaces from WIT definitions for use in RPC clients workers.",
    long_about = None
)]
struct Args {
    /// WIT file to read
    #[arg(short, long)]
    input: String,
    /// Rust file to write
    #[arg(short, long)]
    output: String,
}

pub fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    println!("Parsing schema from '{}'.", args.input);
    let source = expand_wit_source(&args.input)?;
    println!("Writing Rust code to '{}'.", args.output);
    std::fs::write(&args.output, &source)
        .with_context(|| format!("Failed to write generated code to {}", args.output))?;
    Ok(())
}
