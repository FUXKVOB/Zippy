#[derive(Debug)]
pub struct ParsedComponent {
    pub template: String,
    pub script: String,
    pub style: String,
}

pub fn parse(source: &str) -> Result<ParsedComponent, String> {
    let template = extract_section(source, "template")
        .ok_or("Missing <template> section")?;
    let script = extract_section(source, "script").unwrap_or_default();
    let style = extract_section(source, "style").unwrap_or_default();

    Ok(ParsedComponent { template, script, style })
}

fn extract_section(source: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);

    let start = source.find(&open)?;
    let content_start = source[start..].find('>')? + start + 1;
    let end = source[content_start..].find(&close)? + content_start;

    Some(source[content_start..end].trim().to_string())
}
