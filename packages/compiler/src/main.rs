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

    let sourcemap = args.iter().any(|a| a == "--sourcemap" || a == "-m");

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

    let final_output = if sourcemap {
        let map = build_source_map(input_path, &source, &component, &output);
        let json = serde_json::to_string(&map).unwrap_or_default();
        let b64 = base64_encode(&json);
        format!("{}\n//# sourceMappingURL=data:application/json;base64,{}\n", output, b64)
    } else {
        output
    };

    fs::write(&output_path, final_output).expect("Failed to write output");
    
    let dts_path = output_path.replace(ext, ".d.ts");
    fs::write(&dts_path, types).expect("Failed to write .d.ts file");

    println!("Compiled {} -> {} and {}", input_path, output_path, dts_path);
}

fn build_source_map(input_path: &str, source: &str, _component: &parser::ParsedComponent, output: &str) -> serde_json::Value {
    use serde_json::json;
    let _template_line = count_lines_before(source, "<template") as u32;
    let output_lines = output.lines().count() as u32;
    let mappings = format!(";AAAA{};AAAA", "AACA".repeat(output_lines as usize));

    json!({
        "version": 3,
        "file": Path::new(input_path).file_name().unwrap().to_str().unwrap(),
        "sources": [input_path],
        "sourcesContent": [source],
        "names": [],
        "mappings": mappings
    })
}

fn count_lines_before(haystack: &str, needle: &str) -> usize {
    if let Some(pos) = haystack.find(needle) {
        haystack[..pos].lines().count()
    } else {
        0
    }
}

fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[((b0 & 0x03) << 4 | b1 >> 4) as usize] as char);
        if chunk.len() > 1 {
            out.push(ALPHABET[((b1 & 0x0f) << 2 | b2 >> 6) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(ALPHABET[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
