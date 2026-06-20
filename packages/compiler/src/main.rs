mod parser;
mod template;
mod codegen;

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: zippy-compile <input.zippy> [output.js]");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let source = fs::read_to_string(input_path)
        .expect("Failed to read input file");

    let component = parser::parse(&source).expect("Failed to parse .zippy file");
    let ext = if component.script_lang == "ts" { ".ts" } else { ".js" };
    let output = codegen::generate_with_lang(&component.script, &component.template, &component.style, &component.script_lang);

    let output_path = args.get(2)
        .map(|p| p.clone())
        .unwrap_or_else(|| {
            let stem = Path::new(input_path).file_stem().unwrap().to_str().unwrap();
            format!("{}{}", stem, ext)
        });

    fs::write(&output_path, output).expect("Failed to write output");
    println!("Compiled {} -> {}", input_path, output_path);
}
