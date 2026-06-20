use crate::template::{self, AttrValue, Node};

pub fn generate(script: &str, template: &str, style: &str) -> String {
    let nodes = template::parse_template(template).expect("Failed to parse template");
    let info = extract_imports(script);
    let hash = compute_hash(template);
    let scoped_style = scope_css(style, &hash);
    let mut gen = Gen::new(&info.names, &hash);

    let root_html = gen.render(&nodes, Mode::Normal);

    format!(
r#"{imports}
import {{ signal, effect }} from "@zippy/runtime";

export default function ZippyComponent(props) {{
  {body_script}

  const el = document.createElement('div');
  el.setAttribute('data-z-{hash}', '');
{style_setup}

  el.innerHTML = `{root_html}`;

{decls}
{wiring}

  return {{
    el,
    mount(target) {{ target.appendChild(el); }},
    unmount() {{ el.remove();{unmount_comp}{style_teardown} }},
    update(newProps) {{ Object.assign(props, newProps); }},
  }};
}}
"#,
        imports = info.imports,
        body_script = info.body,
        hash = hash,
        style_setup = render_style(&scoped_style),
        style_teardown = if scoped_style.is_empty() { String::new() } else { "\n    __style.remove();".into() },
        root_html = root_html,
        decls = gen.render_decls(),
        wiring = gen.render_wiring(),
        unmount_comp = gen.render_unmount_comp(),
    )
}

// ---------------------------------------------------------------------------
// Import extraction
// ---------------------------------------------------------------------------

struct ImportInfo {
    imports: String,
    body: String,
    names: Vec<String>,
}

fn extract_imports(script: &str) -> ImportInfo {
    let mut imports = Vec::new();
    let mut body = Vec::new();
    let mut names = Vec::new();

    for line in script.lines() {
        let t = line.trim();
        if t.starts_with("import ") {
            let rewritten = t.replace(".zippy", ".js");
            if let Some(rest) = t.strip_prefix("import ") {
                if let Some(name) = rest.split_whitespace().next() {
                    names.push(name.to_string());
                }
            }
            imports.push(rewritten);
        } else {
            body.push(line);
        }
    }

    ImportInfo { imports: imports.join("\n"), body: body.join("\n"), names }
}

// ---------------------------------------------------------------------------
// Hash & CSS scoping
// ---------------------------------------------------------------------------

fn compute_hash(s: &str) -> String {
    let mut h = 5381u64;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    format!("{:x}", h & 0xFFFF_FFFF)
}

fn scope_css(css: &str, hash: &str) -> String {
    if css.is_empty() { return String::new(); }
    let attr = format!("[data-z-{}]", hash);
    let mut out = String::new();
    for line in css.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('}') {
            out.push_str(line);
            out.push('\n');
            continue;
        }
            if trimmed.contains('{') || trimmed.ends_with(',') {
            let prefix = trimmed.trim_end_matches(|c| c == '{' || c == ',');
            let suffix = &trimmed[prefix.len()..];
            let prefixed = prefix.split(',')
                .map(|sel| format!("{} {}", attr, sel.trim()))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!("{}{}{}", "  ".repeat(line.len() - line.trim_start().len()), prefixed, suffix));
            out.push('\n');
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

#[derive(Clone, Copy)]
enum Mode { Normal, Inline, Raw }

fn is_ident(s: &str) -> bool {
    if s.is_empty() { return false; }
    let first = s.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '$' { return false; }
    s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

fn wrap_val(expr: &str) -> String {
    if is_ident(expr) {
        format!("{}.val", expr)
    } else {
        expr.to_string()
    }
}

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

struct Gen {
    component_names: Vec<String>,
    hash: String,
    events: Vec<(String, String)>,
    binds: Vec<(String, String)>,
    exprs: Vec<String>,
    comps: Vec<CompInfo>,
    ifs: Vec<IfInfo>,
    eachs: Vec<EachInfo>,
}

struct CompInfo {
    name: String,
    static_props: Vec<(String, String)>,
    dynamic_props: Vec<(String, String)>,
}

struct IfInfo {
    expr: String,
    idx: usize,
}

struct EachInfo {
    list: String,
    item: String,
    index: Option<String>,
    idx: usize,
    body_html: String,
}

impl Gen {
    fn new(component_names: &[String], hash: &str) -> Self {
        Self {
            component_names: component_names.to_vec(),
            hash: hash.to_string(),
            events: Vec::new(),
            binds: Vec::new(),
            exprs: Vec::new(),
            comps: Vec::new(),
            ifs: Vec::new(),
            eachs: Vec::new(),
        }
    }

    /// Render AST to HTML string.
    /// - `Normal`: expressions → `<span data-zippy-expr="N">` + effects
    /// - `Inline`: expressions → `${{expr}.val}` (dynamic attrs)
    /// - `Raw`:    expressions → `${{expr}}`  (each body — loop vars are plain values)
    fn render(&mut self, nodes: &[Node], mode: Mode) -> String {
        let mut html = String::new();

        for n in nodes {
            match n {
                Node::Element { tag, attrs, children } => {
                    if self.component_names.contains(tag) {
                        let mut ci = CompInfo {
                            name: tag.clone(),
                            static_props: Vec::new(),
                            dynamic_props: Vec::new(),
                        };
                        for a in attrs {
                            match &a.value {
                                AttrValue::Static(v) => ci.static_props.push((a.name.clone(), v.clone())),
                                AttrValue::Dynamic(e) => ci.dynamic_props.push((a.name.clone(), e.clone())),
                                AttrValue::Event(_, _) => {}
                                AttrValue::Bind(_, _) => {} // bind on components not yet supported
                            }
                        }
                        let idx = self.comps.len();
                        self.comps.push(ci);
                        html.push_str(&format!("<div data-zippy-cmp=\"{}\"></div>", idx));
                    } else {
                        html.push('<');
                        html.push_str(tag);
                        for a in attrs {
                            match &a.value {
                                AttrValue::Static(v) => html.push_str(&format!(" {}=\"{}\"", a.name, v)),
                                AttrValue::Dynamic(e) => {
                                    let v = wrap_val(e);
                                    html.push_str(&format!(" {}=\"${{{}}}\"", a.name, v));
                                }
                                AttrValue::Event(ev, handler) => {
                                    let ei = self.events.len();
                                    self.events.push((ev.clone(), handler.clone()));
                                    html.push_str(&format!(" data-zippy-evt{}", ei));
                                }
                                AttrValue::Bind(prop, expr) => {
                                    let bi = self.binds.len();
                                    self.binds.push((prop.clone(), expr.clone()));
                                    html.push_str(&format!(" data-zippy-bind=\"{}\"", bi));
                                }
                            }
                        }
                        if children.is_empty() && is_void(tag) {
                            html.push_str(" />");
                        } else {
                            html.push('>');
                            html.push_str(&self.render(children, mode));
                            html.push_str("</");
                            html.push_str(tag);
                            html.push('>');
                        }
                    }
                }
                Node::Text(t) => html.push_str(t),
                Node::Expr(e) => {
                    match mode {
                        Mode::Normal => {
                            let ei = self.exprs.len();
                            self.exprs.push(e.clone());
                            html.push_str(&format!("<span data-zippy-expr=\"{}\"></span>", ei));
                        }
                        Mode::Inline => html.push_str(&format!("${{{}}}", wrap_val(e))),
                        Mode::Raw => html.push_str(&format!("${{{}}}", e)),
                    }
                }
                Node::IfBlock { branches, fallback: _ } => {
                    let idx = self.ifs.len();
                    self.ifs.push(IfInfo {
                        expr: branches[0].0.clone(),
                        idx,
                    });
                    let body_html = self.render(&branches[0].1, mode);
                    html.push_str(&format!(
                        "<!--zippy-if-{}--><div data-zippy-if=\"{}\">{}</div>",
                        idx, idx, body_html
                    ));
                }
                Node::EachBlock { list, item, index, body } => {
                    let idx = self.eachs.len();
                    let body_html = self.render(body, Mode::Raw); // loop vars are plain values
                    self.eachs.push(EachInfo {
                        list: list.clone(),
                        item: item.clone(),
                        index: index.clone(),
                        idx,
                        body_html,
                    });
                    html.push_str(&format!(
                        "<!--zippy-each-{}--><div data-zippy-each=\"{}\"></div>",
                        idx, idx
                    ));
                }
            }
        }

        html
    }

    fn render_decls(&self) -> String {
        let mut d = String::new();
        for i in 0..self.comps.len() {
            d.push_str(&format!("  let __cmp{};\n", i));
        }
        for info in &self.ifs {
            d.push_str(&format!("  let __if{};\n", info.idx));
        }
        for info in &self.eachs {
            d.push_str(&format!("  let __each{};\n", info.idx));
        }
        d
    }

    fn render_wiring(&self) -> String {
        let mut code = String::new();

        // Events
        for (i, (ev, handler)) in self.events.iter().enumerate() {
            code.push_str(&format!(
                "  const __btn{} = el.querySelector('[data-zippy-evt{}]');\n  \
                 if (__btn{}) __btn{}.addEventListener('{}', {});\n",
                i, i, i, i, ev, handler
            ));
        }

        // Expression effects
        for (i, expr) in self.exprs.iter().enumerate() {
            code.push_str(&format!(
                "  effect(() => {{\n    \
                   const __n = el.querySelector('[data-zippy-expr=\"{}\"]');\n    \
                   if (__n) __n.textContent = {};\n  \
                 }});\n",
                i, wrap_val(expr)
            ));
        }

        // Components
        for (i, ci) in self.comps.iter().enumerate() {
            let mut init_props: Vec<String> = ci.static_props.iter()
                .map(|(k, v)| format!("{}: \"{}\"", k, v))
                .collect();
            for (k, e) in &ci.dynamic_props {
                init_props.push(format!("{}: {}", k, wrap_val(e)));
            }

            code.push_str(&format!(
                "  const __slot{} = el.querySelector('[data-zippy-cmp=\"{}\"]');\n  \
                 if (__slot{}) {{\n    \
                   __cmp{} = {}({{ {} }});\n    \
                   __cmp{}.mount(__slot{});\n  \
                 }}\n",
                i, i, i, i, ci.name, init_props.join(", "), i, i
            ));

            if !ci.dynamic_props.is_empty() {
                let updates: Vec<String> = ci.dynamic_props.iter()
                    .map(|(k, e)| format!("{}: {}", k, wrap_val(e)))
                    .collect();
                code.push_str(&format!(
                    "  effect(() => {{\n    \
                       if (__cmp{}) __cmp{}.update({{ {} }});\n  \
                     }});\n",
                    i, i, updates.join(", ")
                ));
            }
        }

        // If blocks
        for info in &self.ifs {
            code.push_str(&format!(
                "  const __ifAnchor{} = el.querySelector('[data-zippy-if=\"{}\"]');\n  \
                 if (__ifAnchor{}) {{\n    \
                   effect(() => {{\n      \
                     __ifAnchor{}.hidden = !({});\n    \
                   }});\n  \
                 }}\n",
                info.idx, info.idx, info.idx, info.idx, wrap_val(&info.expr)
            ));
        }

        // Bindings
        for (i, (prop, expr)) in self.binds.iter().enumerate() {
            code.push_str(&format!(
                "  const __bind{} = el.querySelector('[data-zippy-bind=\"{}\"]');\n  \
                 if (__bind{}) {{\n    \
                   __bind{}.{} = {};\n    \
                   __bind{}.addEventListener('input', () => {{ {}.val = __bind{}.{}; }});\n    \
                   effect(() => {{ __bind{}.{} = {}.val; }});\n  \
                 }}\n",
                i, i, i, i, prop, wrap_val(expr),
                i, expr, i, prop,
                i, prop, expr
            ));
        }

        // Each blocks
        for info in &self.eachs {
            let idx = info.idx;
            let fn_name = format!("__each_{}", idx);

            let map_params = match &info.index {
                Some(idx_var) => format!("{}, {}", info.item, idx_var),
                None => info.item.clone(),
            };

            code.push_str(&format!(
                "  const {} = () => {{\n    \
                   const __list = {};\n    \
                   return __list.map(({}) => `{}`).join('');\n  \
                 }};\n",
                fn_name, wrap_val(&info.list), map_params, info.body_html
            ));

            code.push_str(&format!(
                "  effect(() => {{\n    \
                   const __parent = el.querySelector('[data-zippy-each=\"{}\"]');\n    \
                   if (!__parent) return;\n    \
                   __parent.innerHTML = {}();\n  \
                 }});\n",
                idx, fn_name
            ));
        }

        code
    }

    fn render_unmount_comp(&self) -> String {
        if self.comps.is_empty() { return String::new(); }
        let mut code = String::new();
        for i in 0..self.comps.len() {
            code.push_str(&format!("\n    if (__cmp{}) __cmp{}.unmount();", i, i));
        }
        code
    }
}

fn is_void(tag: &str) -> bool {
    matches!(tag, "br" | "hr" | "img" | "input" | "meta" | "link" | "area" | "base" | "col" | "embed" | "source" | "track" | "wbr")
}

fn render_style(s: &str) -> String {
    if s.is_empty() { return String::new(); }
    format!(
        "  const __style = document.createElement('style');\n  \
         __style.textContent = `{}`;\n  \
         document.head.append(__style);",
        s.replace('`', "\\`")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ident() {
        assert!(is_ident("name"));
        assert!(is_ident("_foo"));
        assert!(is_ident("$val"));
        assert!(!is_ident("5"));
        assert!(!is_ident("foo.bar"));
        assert!(!is_ident(""));
        assert!(!is_ident("123abc"));
    }

    #[test]
    fn test_wrap_val() {
        assert_eq!(wrap_val("count"), "count.val");
        assert_eq!(wrap_val("5"), "5");
        assert_eq!(wrap_val("foo.bar"), "foo.bar");
        assert_eq!(wrap_val("name"), "name.val");
    }

    #[test]
    fn test_compute_hash() {
        let h = compute_hash("hello");
        assert!(h.len() <= 8);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_basic() {
        let js = generate(
            "let count = signal(0);",
            "<p>{count}</p>",
            "",
        );
        assert!(js.contains("signal"));
        assert!(js.contains("effect"));
        assert!(js.contains("data-zippy-expr"));
        assert!(js.contains("count.val"));
        assert!(js.contains("export default function ZippyComponent"));
    }

    #[test]
    fn test_generate_bind() {
        let js = generate(
            "let x = signal('');",
            "<input bind:value={x} />",
            "",
        );
        assert!(js.contains("data-zippy-bind"));
        assert!(js.contains("addEventListener('input'"));
        assert!(js.contains("x.val"));
    }

    #[test]
    fn test_generate_each_index() {
        let js = generate(
            "let items = signal([1,2,3]);",
            "{#each items as item, i}<li>{i}: {item}</li>{/each}",
            "",
        );
        // items from each body are rendered Raw (no .val)
        assert!(js.contains("${i}"));
        assert!(js.contains("${item}"));
    }

    #[test]
    fn test_generate_if() {
        let js = generate(
            "let show = signal(true);",
            "{#if show}<p>visible</p>{/if}",
            "",
        );
        assert!(js.contains("hidden"));
        assert!(js.contains("show.val"));
    }

    #[test]
    fn test_style_scoping() {
        let scoped = scope_css("h1 { color: red; }", "abc123");
        assert!(scoped.contains("[data-z-abc123]"));
        assert!(scoped.contains("h1"));
    }

    #[test]
    fn test_extract_imports() {
        let info = extract_imports("import Foo from './Foo.zippy'\nlet x = 1");
        assert_eq!(info.names, vec!["Foo"]);
        assert!(info.imports.contains("Foo"));
        assert!(info.imports.contains(".js"));
        assert!(info.body.contains("x = 1"));
    }
}
