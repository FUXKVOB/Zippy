#[derive(Clone)]
pub struct Stream<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    _pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Stream<'a> {
    fn new(source: &'a str) -> Self {
        Self { chars: source.chars().peekable(), _pos: 0, line: 1, col: 1 }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(c) = c {
            self._pos += 1;
            if c == '\n' { self.line += 1; self.col = 1; }
            else { self.col += 1; }
        }
        c
    }

    fn _pos(&self) -> (usize, usize) { (self.line, self.col) }

    fn err(&self, msg: &str) -> String {
        format!("line {} col {}: {}", self.line, self.col, msg)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' { self.next(); }
            else { break; }
        }
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        if self.peek() == Some('@') { s.push(self.next().unwrap()); }
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == ':' { s.push(self.next().unwrap()); }
            else { break; }
        }
        s
    }
}

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
        key: Option<String>,
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
    Bind(String, String),
    ClassToggle(String, String),
}

pub fn parse_template(source: &str) -> Result<Vec<Node>, String> {
    let mut stream = Stream::new(source);
    parse_nodes(&mut stream, &[])
}

fn parse_nodes(stream: &mut Stream, closing: &[&str]) -> Result<Vec<Node>, String> {
    let mut nodes = Vec::new();

    loop {
        stream.skip_ws();
        match stream.peek() {
            None => break,
            Some('<') => {
                stream.next();
                if stream.peek() == Some('/') {
                    stream.next();
                    let mut tag = String::new();
                    while let Some(c) = stream.peek() {
                        if c == '>' { stream.next(); break; }
                        tag.push(c); stream.next();
                    }
                    if closing.contains(&tag.as_str()) {
                        break;
                    }
                } else if stream.peek() == Some('!') {
                    stream.next();
                    if stream.peek() == Some('-') {
                        stream.next(); stream.next();
                        let mut s = String::new();
                        loop {
                            match stream.next() {
                                None => return Err(stream.err("Unclosed <!-- comment")),
                                Some('-') if stream.peek() == Some('-') && stream.clone().next() == Some('>') => {
                                    stream.next(); stream.next(); break;
                                }
                                Some(c) => s.push(c),
                            }
                        }
                        nodes.push(Node::Text(format!("<!--{}-->", s)));
                    } else {
                        return Err(stream.err("Unexpected <! in template"));
                    }
                } else {
                    nodes.push(parse_element(stream)?);
                }
            }
            Some('{') => {
                let saved = stream.clone();
                stream.next();
                match stream.peek() {
                    Some('#') => {
                        stream.next();
                        let mut kw = String::new();
                        while let Some(c) = stream.peek() {
                            if c == ' ' || c == '}' { break; }
                            kw.push(c); stream.next();
                        }
                        match kw.as_str() {
                            "if" => nodes.push(parse_if_block(stream)?),
                            "each" => nodes.push(parse_each_block(stream)?),
                            _ => return Err(stream.err(&format!("Unknown block: #{}", kw))),
                        }
                    }
                    Some(':') => {
                        *stream = saved;
                        break;
                    }
                    Some('/') => {
                        *stream = saved;
                        break;
                    }
                    _ => {
                        let mut expr = String::new();
                        let mut depth = 1;
                        while let Some(c) = stream.next() {
                            match c {
                                '{' => depth += 1,
                                '}' => { depth -= 1; if depth == 0 { break; } }
                                c => expr.push(c),
                            }
                        }
                        if depth != 0 { return Err(stream.err("Unclosed {")); }
                        nodes.push(Node::Expr(expr.trim().into()));
                    }
                }
            }
            Some(_) => {
                nodes.push(parse_text(stream));
            }
        }
    }

    Ok(nodes)
}

fn parse_element(stream: &mut Stream) -> Result<Node, String> {
    let tag = stream.read_ident();
    let mut attrs = Vec::new();

    loop {
        stream.skip_ws();
        match stream.peek() {
            None | Some('>') => { stream.next(); break; }
            Some('/') => {
                stream.next();
                if stream.next() != Some('>') { return Err(stream.err("Expected />")); }
                return Ok(Node::Element { tag, attrs, children: vec![] });
            }
            _ => { attrs.push(parse_attr(stream)?); }
        }
    }

    if is_void(&tag) { return Ok(Node::Element { tag, attrs, children: vec![] }); }

    let children = parse_nodes(stream, &[&tag])?;
    Ok(Node::Element { tag, attrs, children })
}

fn parse_attr(stream: &mut Stream) -> Result<Attr, String> {
    let name = stream.read_ident();

    if name.starts_with('@') {
        let event_name = name[1..].to_string();
        stream.skip_ws();
        if stream.peek() == Some('=') {
            stream.next();
            if let AttrValue::Dynamic(handler) = parse_attr_value(stream) {
                return Ok(Attr { name: format!("data-zippy-on-{}", event_name), value: AttrValue::Event(event_name, handler) });
            }
            return Err(stream.err("@event needs {{handler}}"));
        }
        return Err(stream.err(&format!("@{} requires ={{fn}}", event_name)));
    }

    if name.starts_with("bind:") {
        let prop = name[5..].to_string();
        stream.skip_ws();
        if stream.peek() == Some('=') {
            stream.next();
            if let AttrValue::Dynamic(expr) = parse_attr_value(stream) {
                return Ok(Attr { name, value: AttrValue::Bind(prop, expr) });
            }
        }
        return Err(stream.err(&format!("bind:{} requires ={{expr}}", prop)));
    }

    if name.starts_with("class:") {
        let class_name = name[6..].to_string();
        stream.skip_ws();
        if stream.peek() == Some('=') {
            stream.next();
            if let AttrValue::Dynamic(expr) = parse_attr_value(stream) {
                return Ok(Attr { name, value: AttrValue::ClassToggle(class_name, expr) });
            }
        }
        return Err(stream.err(&format!("class:{} requires ={{expr}}", class_name)));
    }

    stream.skip_ws();
    if stream.peek() == Some('=') {
        stream.next();
        Ok(Attr { name, value: parse_attr_value(stream) })
    } else {
        Ok(Attr { name, value: AttrValue::Static(String::new()) })
    }
}

fn parse_attr_value(stream: &mut Stream) -> AttrValue {
    match stream.peek() {
        Some('"') => {
            stream.next();
            let mut s = String::new();
            loop {
                match stream.next() {
                    None | Some('"') => break,
                    Some(c) => s.push(c),
                }
            }
            AttrValue::Static(s)
        }
        Some('{') => {
            stream.next();
            let mut expr = String::new();
            let mut depth = 1;
            while let Some(c) = stream.next() {
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
            while let Some(c) = stream.peek() {
                if c == '>' || c == '/' || c == ' ' || c == '\t' || c == '\n' { break; }
                s.push(c); stream.next();
            }
            AttrValue::Static(s)
        }
    }
}

fn parse_if_block(stream: &mut Stream) -> Result<Node, String> {
    stream.skip_ws();
    let mut cond = String::new();
    while let Some(c) = stream.peek() {
        if c == '}' { stream.next(); break; }
        cond.push(c); stream.next();
    }

    let mut branches: Vec<(String, Vec<Node>)> = Vec::new();
    let then_body = parse_nodes(stream, &["/if", ":else", ":else if"])?;
    branches.push((cond.trim().to_string(), then_body));

    let mut fallback: Vec<Node> = Vec::new();

    loop {
        match stream.peek() {
            None => return Err(stream.err("Unclosed {#if}")),
            Some('{') => {
                stream.next();
                match stream.peek() {
                    Some(':') => {
                        stream.next();
                        let mut kw = String::new();
                        while let Some(c) = stream.peek() {
                            if c == '}' { stream.next(); break; }
                            kw.push(c); stream.next();
                        }
                        let kw = kw.trim();
                        if kw == "else" {
                            fallback = parse_nodes(stream, &["/if"])?;
                            if stream.peek() == Some('{') { stream.next(); }
                            while let Some(c) = stream.peek() {
                                if c == '}' { stream.next(); break; }
                                stream.next();
                            }
                            break;
                        } else if kw.starts_with("else if") {
                            let cond = kw[7..].trim().to_string();
                            let body = parse_nodes(stream, &["/if", ":else", ":else if"])?;
                            branches.push((cond, body));
                            continue;
                        } else {
                            return Err(stream.err(&format!("Unknown block keyword: {}", kw)));
                        }
                    }
                    Some('/') => {
                        stream.next();
                        let mut end = String::new();
                        while let Some(c) = stream.peek() {
                            if c == '}' { stream.next(); break; }
                            end.push(c); stream.next();
                        }
                        if end.trim() != "if" { return Err(stream.err("Expected {/if}")); }
                        break;
                    }
                    _ => return Err(stream.err("Expected {:else} or {/if}")),
                }
            }
            Some(_) => break,
            None => break,
        }
    }

    Ok(Node::IfBlock { branches, fallback })
}

fn parse_each_block(stream: &mut Stream) -> Result<Node, String> {
    stream.skip_ws();
    let mut rest = String::new();
    while let Some(c) = stream.peek() {
        if c == '}' { stream.next(); break; }
        rest.push(c); stream.next();
    }

    let parts: Vec<&str> = rest.splitn(3, ' ').collect();
    if parts.len() < 3 || parts[1] != "as" {
        return Err(stream.err("{#each items as item} expected"));
    }
    let list = parts[0].trim().to_string();
    let rest2 = parts[2..].join(" ");
    let (rest2, key) = if let Some(key_pos) = rest2.find(" key=") {
        let before_key = rest2[..key_pos].trim().to_string();
        let key_part = rest2[key_pos + 5..].trim();
        let key = if key_part.starts_with('{') && key_part.ends_with('}') {
            Some(key_part[1..key_part.len()-1].trim().to_string())
        } else {
            return Err(stream.err("key={expr} expected"));
        };
        (before_key, key)
    } else {
        (rest2, None)
    };
    let (item, index) = if let Some(pos) = rest2.find(',') {
        (rest2[..pos].trim().to_string(), Some(rest2[pos+1..].trim().to_string()))
    } else {
        (rest2.trim().to_string(), None)
    };

    let body = parse_nodes(stream, &["/each"])?;
    if stream.peek() == Some('{') {
        stream.next();
        if stream.peek() == Some('/') {
            stream.next();
            while let Some(c) = stream.peek() {
                if c == '}' { stream.next(); break; }
                stream.next();
            }
        }
    }

    Ok(Node::EachBlock { list, item, index, key, body })
}

fn parse_text(stream: &mut Stream) -> Node {
    let mut s = String::new();
    while let Some(c) = stream.peek() {
        if c == '<' || c == '{' { break; }
        s.push(c); stream.next();
    }
    Node::Text(s)
}

fn is_void(tag: &str) -> bool {
    matches!(tag, "br" | "hr" | "img" | "input" | "meta" | "link" | "area" | "base" | "col" | "embed" | "source" | "track" | "wbr")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_element() {
        let nodes = parse_template("<div></div>").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::Element { tag, attrs, children } => {
                assert_eq!(tag, "div");
                assert!(attrs.is_empty());
                assert!(children.is_empty());
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_element_with_text() {
        let nodes = parse_template("<p>hello</p>").unwrap();
        match &nodes[0] {
            Node::Element { tag, children, .. } => {
                assert_eq!(tag, "p");
                assert_eq!(children.len(), 1);
                match &children[0] {
                    Node::Text(t) => assert_eq!(t, "hello"),
                    _ => panic!("expected Text"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_expression() {
        let nodes = parse_template("<span>{count}</span>").unwrap();
        match &nodes[0] {
            Node::Element { children, .. } => {
                match &children[0] {
                    Node::Expr(e) => assert_eq!(e, "count"),
                    _ => panic!("expected Expr"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_static_attr() {
        let nodes = parse_template(r#"<div class="foo"></div>"#).unwrap();
        match &nodes[0] {
            Node::Element { attrs, .. } => {
                assert_eq!(attrs.len(), 1);
                assert_eq!(attrs[0].name, "class");
                match &attrs[0].value {
                    AttrValue::Static(v) => assert_eq!(v, "foo"),
                    _ => panic!("expected Static"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_dynamic_attr() {
        let nodes = parse_template("<div class={name}></div>").unwrap();
        match &nodes[0] {
            Node::Element { attrs, .. } => {
                assert_eq!(attrs[0].name, "class");
                match &attrs[0].value {
                    AttrValue::Dynamic(e) => assert_eq!(e, "name"),
                    _ => panic!("expected Dynamic"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_event_attr() {
        let nodes = parse_template("<button @click={fn}></button>").unwrap();
        match &nodes[0] {
            Node::Element { attrs, .. } => {
                match &attrs[0].value {
                    AttrValue::Event(ev, handler) => {
                        assert_eq!(ev, "click");
                        assert_eq!(handler, "fn");
                    }
                    _ => panic!("expected Event"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_bind_attr() {
        let nodes = parse_template("<input bind:value={x} />").unwrap();
        match &nodes[0] {
            Node::Element { tag, attrs, .. } => {
                assert_eq!(tag, "input");
                match &attrs[0].value {
                    AttrValue::Bind(prop, expr) => {
                        assert_eq!(prop, "value");
                        assert_eq!(expr, "x");
                    }
                    _ => panic!("expected Bind"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_self_closing() {
        let nodes = parse_template("<br /><hr />").unwrap();
        assert_eq!(nodes.len(), 2);
        for n in &nodes {
            match n {
                Node::Element { children, .. } => assert!(children.is_empty()),
                _ => panic!("expected Element"),
            }
        }
    }

    #[test]
    fn parse_if_block() {
        let nodes = parse_template("{#if show}<p>hi</p>{/if}").unwrap();
        assert_eq!(nodes.len(), 1);
        match &nodes[0] {
            Node::IfBlock { branches, fallback } => {
                assert_eq!(branches.len(), 1);
                assert_eq!(branches[0].0, "show");
                assert!(fallback.is_empty());
            }
            _ => panic!("expected IfBlock"),
        }
    }

    #[test]
    fn parse_if_else() {
        let nodes = parse_template("{#if a}<p>a</p>{:else}<p>b</p>{/if}").unwrap();
        match &nodes[0] {
            Node::IfBlock { branches, fallback } => {
                assert_eq!(branches.len(), 1);
                assert!(!fallback.is_empty());
            }
            _ => panic!("expected IfBlock"),
        }
    }

    #[test]
    fn parse_each_block() {
        let nodes = parse_template("{#each items as item}<li>{item}</li>{/each}").unwrap();
        match &nodes[0] {
            Node::EachBlock { list, item, index, .. } => {
                assert_eq!(list, "items");
                assert_eq!(item, "item");
                assert!(index.is_none());
            }
            _ => panic!("expected EachBlock"),
        }
    }

    #[test]
    fn parse_each_with_index() {
        let nodes = parse_template("{#each items as item, i}<li>{i}: {item}</li>{/each}").unwrap();
        match &nodes[0] {
            Node::EachBlock { list, item, index, .. } => {
                assert_eq!(list, "items");
                assert_eq!(item, "item");
                assert_eq!(index.as_deref(), Some("i"));
            }
            _ => panic!("expected EachBlock"),
        }
    }

    #[test]
    fn parse_nested_elements() {
        let nodes = parse_template("<ul><li>a</li><li>b</li></ul>").unwrap();
        match &nodes[0] {
            Node::Element { tag, children, .. } => {
                assert_eq!(tag, "ul");
                assert_eq!(children.len(), 2);
                match &children[0] {
                    Node::Element { tag, .. } => assert_eq!(tag, "li"),
                    _ => panic!("expected Element"),
                }
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_multiple_attrs() {
        let nodes = parse_template(r#"<div class="x" id={y} @click={z}></div>"#).unwrap();
        match &nodes[0] {
            Node::Element { attrs, .. } => {
                assert_eq!(attrs.len(), 3);
            }
            _ => panic!("expected Element"),
        }
    }

    #[test]
    fn parse_error_unclosed_expr() {
        let result = parse_template("<p>{unclosed</p>");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_line_number() {
        let result = parse_template("<div>\n<span>{bad\n</span>\n</div>");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("line 4"), "error should contain line number, got: {}", err);
    }
}
