use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug)]
pub enum Node {
    Element {
        tag: String,
        attrs: Vec<Attr>,
        children: Vec<Node>,
    },
    Text(String),
    Expr(String),
    IfBlock {
        branches: Vec<(String, Vec<Node>)>,
        fallback: Vec<Node>,
    },
    EachBlock {
        list: String,
        item: String,
        index: Option<String>,
        body: Vec<Node>,
    },
}

#[derive(Debug)]
pub struct Attr {
    pub name: String,
    pub value: AttrValue,
}

#[derive(Debug)]
pub enum AttrValue {
    Static(String),
    Dynamic(String),
    Event(String, String),
}

pub fn parse_template(source: &str) -> Result<Vec<Node>, String> {
    let mut chars = source.chars().peekable();
    parse_nodes(&mut chars, &[])
}

fn parse_nodes(chars: &mut Peekable<Chars>, closing: &[&str]) -> Result<Vec<Node>, String> {
    let mut nodes = Vec::new();

    loop {
        skip_ws(chars);
        match chars.peek() {
            None => break,
            Some('<') => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    // closing tag
                    chars.next();
                    let mut tag = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '>' { chars.next(); break; }
                        tag.push(c); chars.next();
                    }
                    if closing.contains(&tag.as_str()) {
                        break;
                    }
                } else if chars.peek() == Some(&'!') {
                    chars.next();
                    if chars.peek() == Some(&'-') {
                        // comment <!-- ... -->
                        let mut s = String::new();
                        loop {
                            match chars.next() {
                                None => break,
                                Some('-') if chars.peek() == Some(&'-') && chars.clone().nth(1) == Some('>') => {
                                    chars.next(); chars.next(); break;
                                }
                                Some(c) => s.push(c),
                            }
                        }
                        nodes.push(Node::Text(format!("<!--{}-->", s)));
                    } else {
                        return Err("Unexpected <! in template".into());
                    }
                } else {
                    nodes.push(parse_element(chars)?);
                }
            }
            Some('{') => {
                let saved = chars.clone();
                chars.next();
                match chars.peek() {
                    Some('#') => {
                        chars.next();
                        let mut kw = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == ' ' || c == '}' { break; }
                            kw.push(c); chars.next();
                        }
                        match kw.as_str() {
                            "if" => nodes.push(parse_if_block(chars)?),
                            "each" => nodes.push(parse_each_block(chars)?),
                            _ => return Err(format!("Unknown block: #{}", kw)),
                        }
                    }
                    Some(':') => {
                        // {:else} or {:else if} — belongs to outer if, stop here
                        *chars = saved;
                        break;
                    }
                    Some('/') => {
                        // {/if} or {/each} — end block
                        *chars = saved;
                        break;
                    }
                    _ => {
                        let mut expr = String::new();
                        let mut depth = 1;
                        while let Some(c) = chars.next() {
                            match c {
                                '{' => depth += 1,
                                '}' => { depth -= 1; if depth == 0 { break; } }
                                c => expr.push(c),
                            }
                        }
                        if depth != 0 { return Err("Unclosed {".into()); }
                        nodes.push(Node::Expr(expr.trim().into()));
                    }
                }
            }
            Some(_) => {
                nodes.push(parse_text(chars));
            }
        }
    }

    Ok(nodes)
}

fn parse_element(chars: &mut Peekable<Chars>) -> Result<Node, String> {
    let tag = read_ident(chars);
    let mut attrs = Vec::new();

    loop {
        skip_ws(chars);
        match chars.peek() {
            None | Some('>') => { chars.next(); break; }
            Some('/') => {
                chars.next();
                if chars.next() != Some('>') { return Err("Expected />".into()); }
                return Ok(Node::Element { tag, attrs, children: vec![] });
            }
            _ => { attrs.push(parse_attr(chars)?); }
        }
    }

    if is_void(&tag) { return Ok(Node::Element { tag, attrs, children: vec![] }); }

    let children = parse_nodes(chars, &[&tag])?;
    Ok(Node::Element { tag, attrs, children })
}

fn parse_attr(chars: &mut Peekable<Chars>) -> Result<Attr, String> {
    let name = read_ident(chars);

    if name.starts_with('@') {
        let event_name = name[1..].to_string();
        skip_ws(chars);
        if chars.peek() == Some(&'=') {
            chars.next();
            if let AttrValue::Dynamic(handler) = parse_attr_value(chars) {
                return Ok(Attr { name: format!("data-zippy-on-{}", event_name), value: AttrValue::Event(event_name, handler) });
            }
            return Err("@event needs {{handler}}".into());
        }
        return Err(format!("@{} requires ={{fn}}", event_name));
    }

    skip_ws(chars);
    if chars.peek() == Some(&'=') {
        chars.next();
        Ok(Attr { name, value: parse_attr_value(chars) })
    } else {
        Ok(Attr { name, value: AttrValue::Static(String::new()) })
    }
}

fn parse_attr_value(chars: &mut Peekable<Chars>) -> AttrValue {
    match chars.peek() {
        Some('"') => {
            chars.next();
            let mut s = String::new();
            loop {
                match chars.next() {
                    None | Some('"') => break,
                    Some(c) => s.push(c),
                }
            }
            AttrValue::Static(s)
        }
        Some('{') => {
            chars.next();
            let mut expr = String::new();
            let mut depth = 1;
            while let Some(c) = chars.next() {
                match c {
                    '{' => depth += 1,
                    '}' => { depth -= 1; if depth == 0 { break; } }
                    c => expr.push(c),
                }
            }
            AttrValue::Dynamic(expr.trim().into())
        }
        _ => {
            let mut s = String::new();
            while let Some(&c) = chars.peek() {
                if c == '>' || c == '/' || c == ' ' || c == '\t' || c == '\n' { break; }
                s.push(c); chars.next();
            }
            AttrValue::Static(s)
        }
    }
}

// ----------------------------------------------------------------
// {#if} block
// ----------------------------------------------------------------
fn parse_if_block(chars: &mut Peekable<Chars>) -> Result<Node, String> {
    skip_ws(chars);
    let mut cond = String::new();
    while let Some(&c) = chars.peek() {
        if c == '}' { chars.next(); break; }
        cond.push(c); chars.next();
    }

    let mut branches: Vec<(String, Vec<Node>)> = Vec::new();
    let then_body = parse_nodes(chars, &["/if", ":else", ":else if"])?;
    branches.push((cond.trim().to_string(), then_body));

    let mut fallback: Vec<Node> = Vec::new();

    loop {
        match chars.peek() {
            None => return Err("Unclosed {#if}".into()),
            Some('{') => {
                chars.next();
                match chars.peek() {
                    Some(':') => {
                        chars.next();
                        let mut kw = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == '}' { chars.next(); break; }
                            kw.push(c); chars.next();
                        }
                        let kw = kw.trim();
                        if kw == "else" {
                            fallback = parse_nodes(chars, &["/if"])?;
                            chars.next(); chars.next(); chars.next(); // consume {/if}
                            break;
                        } else if kw.starts_with("else if") {
                            let cond = kw[7..].trim().to_string();
                            let body = parse_nodes(chars, &["/if", ":else", ":else if"])?;
                            branches.push((cond, body));
                            continue;
                        } else {
                            return Err(format!("Unknown block keyword: {}", kw));
                        }
                    }
                    Some('/') => {
                        chars.next(); // /
                        let mut end = String::new();
                        while let Some(&c) = chars.peek() {
                            if c == '}' { chars.next(); break; }
                            end.push(c); chars.next();
                        }
                        if end.trim() != "if" { return Err("Expected {/if}".into()); }
                        break;
                    }
                    _ => return Err("Expected {:else} or {/if}".into()),
                }
            }
            Some(_) => {
                // shouldn't reach here since parse_nodes consumes delimiters
                break;
            }
            None => break,
        }
    }

    Ok(Node::IfBlock { branches, fallback })
}

// ----------------------------------------------------------------
// {#each} block
// ----------------------------------------------------------------
fn parse_each_block(chars: &mut Peekable<Chars>) -> Result<Node, String> {
    skip_ws(chars);
    let mut rest = String::new();
    while let Some(&c) = chars.peek() {
        if c == '}' { chars.next(); break; }
        rest.push(c); chars.next();
    }

    // rest: "items as item" or "items as item, index"
    let parts: Vec<&str> = rest.splitn(3, ' ').collect();
    if parts.len() < 3 || parts[1] != "as" {
        return Err("{#each items as item} expected".into());
    }
    let list = parts[0].trim().to_string();
    let rest2 = parts[2..].join(" ");
    let (item, index) = if let Some(pos) = rest2.find(',') {
        (rest2[..pos].trim().to_string(), Some(rest2[pos+1..].trim().to_string()))
    } else {
        (rest2.trim().to_string(), None)
    };

    let body = parse_nodes(chars, &["/each"])?;
    // consume {/each}
    if chars.peek() == Some(&'{') {
        chars.next();
        if chars.peek() == Some(&'/') {
            chars.next();
            while let Some(&c) = chars.peek() {
                if c == '}' { chars.next(); break; }
                chars.next();
            }
        }
    }

    Ok(Node::EachBlock { list, item, index, body })
}

fn parse_text(chars: &mut Peekable<Chars>) -> Node {
    let mut s = String::new();
    while let Some(&c) = chars.peek() {
        if c == '<' || c == '{' { break; }
        s.push(c); chars.next();
    }
    Node::Text(s)
}

fn read_ident(chars: &mut Peekable<Chars>) -> String {
    let mut s = String::new();
    if chars.peek() == Some(&'@') { s.push(chars.next().unwrap()); }
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' || c == '-' || c == ':' { s.push(chars.next().unwrap()); }
        else { break; }
    }
    s
}

fn skip_ws(chars: &mut Peekable<Chars>) {
    while let Some(&c) = chars.peek() {
        if c == ' ' || c == '\t' || c == '\n' || c == '\r' { chars.next(); }
        else { break; }
    }
}

fn is_void(tag: &str) -> bool {
    matches!(tag, "br" | "hr" | "img" | "input" | "meta" | "link" | "area" | "base" | "col" | "embed" | "source" | "track" | "wbr")
}
