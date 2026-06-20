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

    let component = match parser::parse(&source) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing .zippy file: {}", e);
            std::process::exit(1);
        }
    };
    let ext = if component.script_lang == "ts" { ".ts" } else { ".js" };
    let (output, types) = match codegen::generate_with_lang(&component.script, &component.template, &component.style, &component.script_lang) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("Error generating code: {}", e);
            std::process::exit(1);
        }
    };

    let output_path = args.get(2)
        .map(|p| p.clone())
        .unwrap_or_else(|| {
            let stem = Path::new(input_path).file_stem().unwrap().to_str().unwrap();
            format!("{}{}", stem, ext)
        });

    fs::write(&output_path, output).expect("Failed to write output");
    
    let dts_path = output_path.replace(ext, ".d.ts");
    fs::write(&dts_path, types).expect("Failed to write .d.ts file");

    println!("Compiled {} -> {} and {}", input_path, output_path, dts_path);
}
