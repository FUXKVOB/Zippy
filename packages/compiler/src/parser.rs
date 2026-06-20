#[derive(Debug)]
pub struct ParsedComponent {
    pub template: String,
    pub script: String,
    pub style: String,
    pub script_lang: String,
}

pub fn parse(source: &str) -> Result<ParsedComponent, String> {
    let template = extract_section(source, "template")
        .ok_or("Missing <template> section")?;
    let script = extract_section(source, "script").unwrap_or_default();
    let style = extract_section(source, "style").unwrap_or_default();
    let script_lang = detect_script_lang(source);

    Ok(ParsedComponent { template, script, style, script_lang })
}

fn detect_script_lang(source: &str) -> String {
    if let Some(start) = source.find("<script") {
        let rest = &source[start..start + 40.min(source.len() - start)];
        if rest.contains("lang=\"ts\"") || rest.contains("lang='ts'") || rest.contains("lang=ts") {
            return "ts".into();
        }
    }
    "js".into()
}

fn extract_section(source: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let start = source.find(&open)?;
    let content_start = source[start..].find('>')? + start + 1;
    let end = source[content_start..].find(&close)? + content_start;

    Some(source[content_start..end].trim().to_string())
}
