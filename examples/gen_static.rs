//! Generate static HTML from .oui files
//!
//! Usage: cargo run --example gen_static -- <input.oui> <output.html> <title>

use oxide_compiler::{compile_file, generate_html};
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: {} <input.oui> <output.html> <title>", args[0]);
        std::process::exit(1);
    }

    let input = Path::new(&args[1]);
    let output = Path::new(&args[2]);
    let title = &args[3];

    println!("Compiling {} ...", input.display());
    let ir = compile_file(input).expect("Failed to compile .oui file");

    println!("Generating HTML...");
    let html = generate_html(&ir, title);

    println!("Writing to {} ...", output.display());
    fs::write(output, html).expect("Failed to write output");

    println!("Done! Generated static HTML.");
}
