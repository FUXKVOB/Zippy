use crate::template::{self, AttrValue, Node};

    #[allow(dead_code)]
    pub fn generate(script: &str, template: &str, style: &str) -> Result<(String, String), String> {
        generate_with_lang(script, template, style, "js")
    }

pub fn generate_with_lang(script: &str, template: &str, style: &str, lang: &str) -> Result<(String, String), String> {
    let ext = if lang == "ts" { ".ts" } else { ".js" };
    let nodes = template::parse_template(template)?;
    let info = extract_imports(script, ext);
    let hash = compute_hash(template);
    let scoped_style = scope_css(style, &hash);
    let mut gen = Gen::new(&info.names, &hash);

    let root_js = gen.render_to_js(&nodes, "el");

    let runtime_imports = if gen.has_each() {
        r#"import { signal, effect, on, reconcileEach, clearAfter } from "@zippy/runtime";"#
    } else if gen.has_events() {
        r#"import { signal, effect, on, clearAfter } from "@zippy/runtime";"#
    } else {
        r#"import { signal, effect, clearAfter } from "@zippy/runtime";"#
    };

    let js = format!(
r#"{imports}
{runtime_imports}

export default function ZippyComponent(props) {{
  const __onMount = [];
  const __onDestroy = [];
  function onMount(fn) {{ __onMount.push(fn); }}
  function onDestroy(fn) {{ __onDestroy.push(fn); }}

  {body_script}

  const el = document.createElement('div');
  el.setAttribute('data-z-{hash}', '');
{style_setup}

{root_js}

  return {{
    el,
    mount(target) {{ target.appendChild(el); __onMount.forEach(fn => fn()); }},
    unmount() {{ el.remove(); __onDestroy.forEach(fn => fn());{unmount_comp}{style_teardown} }},
    update(newProps) {{ Object.assign(props, newProps); }},
  }};
}}
"#,
        imports = info.imports,
        runtime_imports = runtime_imports,
        body_script = info.body,
        hash = hash,
        style_setup = render_style(&scoped_style),
        style_teardown = if scoped_style.is_empty() { String::new() } else { "\n    __style.remove();".into() },
        root_js = root_js,
        unmount_comp = gen.render_unmount_comp(),
    );

    let types = generate_types(&info.body);

    Ok((js, types))
}

fn generate_types(body: &str) -> String {
    let mut props = Vec::new();
    for line in body.lines() {
        if let Some(start) = line.find("props.") {
            let rest = &line[start + 6..];
            let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("");
            if !name.is_empty() && !props.contains(&name.to_string()) {
                props.push(name.to_string());
            }
        }
    }

    let props_def = if props.is_empty() {
        "".to_string()
    } else {
        props.iter()
            .map(|p| format!("  {}: any;", p))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
r#"import {{ ZippyComponent }} from "@zippy/runtime";

export interface Props {{
{props_def}
}}

declare const Component: (props: Props) => ZippyComponent;
export default Component;
"#)
}


// ---------------------------------------------------------------------------
// Import extraction
// ---------------------------------------------------------------------------

struct ImportInfo {
    imports: String,
    body: String,
    names: Vec<String>,
}

fn extract_imports(script: &str, ext: &str) -> ImportInfo {
    let mut imports = Vec::new();
    let mut body = Vec::new();
    let mut names = Vec::new();

    for line in script.lines() {
        let t = line.trim();
        if t.starts_with("import ") {
            let rewritten = t.replace(".zippy", ext);
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
    
    // Simple but more robust approach: 
    // We split by '}' to get rule blocks, then handle each selector.
    for block in css.split('}') {
        let parts: Vec<&str> = block.splitn(2, '{').collect();
        if parts.len() < 2 { 
            if !block.trim().is_empty() {
                out.push_str(block);
                out.push('}');
            }
            continue; 
        }
        
        let selectors = parts[0].trim();
        let body = parts[1];
        
        let scoped_selectors = selectors.split(',')
            .map(|s| {
                let s = s.trim();
                if s.is_empty() { return "".to_string(); }
                // If it starts with @ (at-rule), don't scope
                if s.starts_with('@') { return s.to_string(); }
                format!("{} {}", attr, s)
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        
        out.push_str(&format!("{} {{ {}}} \n", scoped_selectors, body.trim()));
    }
    out
}

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

#[allow(dead_code)]
struct Gen {
    component_names: Vec<String>,
    hash: String,
    events_count: usize,
    comps: Vec<CompInfo>,
    ifs: Vec<IfInfo>,
    eachs: Vec<EachInfo>,
    current_each: Option<usize>,
}

#[allow(dead_code)]
struct CompInfo {
    name: String,
    static_props: Vec<(String, String)>,
    dynamic_props: Vec<(String, String)>,
    children_html: String,
}

#[allow(dead_code)]
struct IfInfo {
    idx: usize,
}

#[allow(dead_code)]
struct EachInfo {
    list: String,
    item: String,
    index: Option<String>,
    key: Option<String>,
    idx: usize,
    events: Vec<(String, String)>,
    binds: Vec<(String, String)>,
    toggles: Vec<(String, String)>,
    exprs: Vec<String>,
}

impl Gen {
    fn new(component_names: &[String], hash: &str) -> Self {
        Self {
            component_names: component_names.to_vec(),
            hash: hash.to_string(),
            events_count: 0,
            comps: Vec::new(),
            ifs: Vec::new(),
            eachs: Vec::new(),
            current_each: None,
        }
    }

    fn render_to_js(&mut self, nodes: &[Node], parent_var: &str) -> String {
        let mut js = String::new();

        for (i, n) in nodes.iter().enumerate() {
            let id = format!("_n{}", i);
            match n {
                Node::Element { tag, attrs, children } => {
                    if tag == "slot" {
                        js.push_str(&format!(
                            "  const {} = document.createElement('div');\n  {}.setAttribute('data-zippy-slot', '');\n  {}.appendChild({});\n",
                            id, id, parent_var, id
                        ));
                        continue;
                    }
                    if self.component_names.contains(tag) {
                        let mut ci = CompInfo {
                            name: tag.clone(),
                            static_props: Vec::new(),
                            dynamic_props: Vec::new(),
                            children_html: String::new(),
                        };
                        for a in attrs {
                            match &a.value {
                                AttrValue::Static(v) => ci.static_props.push((a.name.clone(), v.clone())),
                                AttrValue::Dynamic(e) => ci.dynamic_props.push((a.name.clone(), e.clone())),
                                _ => {}
                            }
                        }
                        let comp_idx = self.comps.len();
                        self.comps.push(ci);

                        // Declare scope level variable for component unmounting
                        js.push_str(&format!("  let __cmp{};\n", comp_idx));

                        // Host element
                        js.push_str(&format!(
                            "  const {} = document.createElement('div');\n  {}.setAttribute('data-zippy-cmp', '{}');\n",
                            id, id, comp_idx
                        ));

                        // Instantiate component
                        let ci = &self.comps[comp_idx];
                        let mut init_props: Vec<String> = ci.static_props.iter()
                            .map(|(k, v)| format!("{}: \"{}\"", k, v))
                            .collect();
                        for (k, e) in &ci.dynamic_props {
                            init_props.push(format!("{}: {}", k, wrap_val(e)));
                        }

                        js.push_str(&format!(
                            "  __cmp{} = {}({{ {} }});\n  __cmp{}.mount({});\n",
                            comp_idx, tag, init_props.join(", "), comp_idx, id
                        ));

                        if !ci.dynamic_props.is_empty() {
                            let updates: Vec<String> = ci.dynamic_props.iter()
                                .map(|(k, e)| format!("{}: {}", k, wrap_val(e)))
                                .collect();
                            js.push_str(&format!(
                                "  effect(() => {{\n    if (__cmp{}) __cmp{}.update({{ {} }});\n  }});\n",
                                comp_idx, comp_idx, updates.join(", ")
                            ));
                        }

                        js.push_str(&format!("  {}.appendChild({});\n", parent_var, id));
                    } else {
                        js.push_str(&format!("  const {} = document.createElement('{}');\n", id, tag));
                        
                        for a in attrs {
                            match &a.value {
                                AttrValue::Static(v) => {
                                    js.push_str(&format!("  {}.setAttribute('{}', '{}');\n", id, a.name, v));
                                }
                                AttrValue::Dynamic(e) => {
                                    js.push_str(&format!(
                                        "  effect(() => {{ {}.setAttribute('{}', {}); }});\n",
                                        id, a.name, wrap_val(e)
                                    ));
                                }
                                AttrValue::Event(ev, handler) => {
                                    self.events_count += 1;
                                    js.push_str(&format!(
                                        "  on({}, '{}', {}, __onDestroy);\n",
                                        id, ev, handler
                                    ));
                                }
                                AttrValue::Bind(prop, expr) => {
                                    js.push_str(&format!(
                                        "  {}.{} = {};\n  {}.addEventListener('input', () => {{ {}.val = {}.{}; }});\n  effect(() => {{ {}.{} = {}.val; }});\n",
                                        id, prop, wrap_val(expr), id, expr, id, prop, id, prop, expr
                                    ));
                                }
                                AttrValue::ClassToggle(class_name, expr) => {
                                    js.push_str(&format!(
                                        "  effect(() => {{ {}.classList.toggle('{}', {}); }});\n",
                                        id, class_name, wrap_val(expr)
                                    ));
                                }
                            }
                        }

                        if !children.is_empty() {
                            js.push_str(&self.render_to_js(children, &id));
                        }
                        js.push_str(&format!("  {}.appendChild({});\n", parent_var, id));
                    }
                }
                Node::Text(t) => {
                    js.push_str(&format!(
                        "  {}.appendChild(document.createTextNode('{}'));\n",
                        parent_var, t.replace('\'', "\\'")
                    ));
                }
                Node::Expr(e) => {
                    js.push_str(&format!(
                        "  const {} = document.createTextNode('');\n  {}.appendChild({});\n  effect(() => {{ {}.textContent = {}; }});\n",
                        id, parent_var, id, id, wrap_val(e)
                    ));
                }
                Node::IfBlock { branches, fallback } => {
                    let if_idx = self.ifs.len();
                    self.ifs.push(IfInfo { idx: if_idx });
                    
                    js.push_str(&format!(
                        "  const {} = document.createComment('zippy-if-{}');\n  {}.appendChild({});\n", 
                        id, if_idx, parent_var, id));
                    
                    js.push_str(&format!(
                        "  let __if_current_branch{} = -1;\n  effect(() => {{\n", id));
                    
                    for (bi, (cond, body)) in branches.iter().enumerate() {
                        let branch_var = format!("__b{}", bi);
                        let cond_val = wrap_val(cond);
                        if bi == 0 {
                            js.push_str(&format!(
                                "    if ({}) {{\n", cond_val));
                        } else {
                            js.push_str(&format!(
                                "    else if ({}) {{\n", cond_val));
                        }
                        
                        js.push_str(&format!(
                            "      if (__if_current_branch{} !== {}) {{\n", id, bi));
                        js.push_str(&format!(
                            "        clearAfter({});\n", id));
                        js.push_str(&format!(
                            "        const {} = document.createDocumentFragment();\n", branch_var));
                        js.push_str(&self.render_to_js(body, &branch_var));
                        js.push_str(&format!(
                            "        {}.appendChild({});\n        __if_current_branch{} = {};\n      }}\n    }}\n", 
                            id, branch_var, id, bi));
                    }
                    
                    if !fallback.is_empty() {
                        let fallback_var = "__bf";
                        js.push_str(&format!(
                            "    else {{\n      if (__if_current_branch{} !== -2) {{\n", id));
                        js.push_str(&format!(
                            "        clearAfter({});\n", id));
                        js.push_str(&format!(
                            "        const {} = document.createDocumentFragment();\n", fallback_var));
                        js.push_str(&self.render_to_js(fallback, fallback_var));
                        js.push_str(&format!(
                            "        {}.appendChild({});\n        __if_current_branch{} = -2;\n      }}\n    }}\n", 
                            id, fallback_var, id));
                    }
                    
                    js.push_str("  });\n");
                }
                Node::EachBlock { list, item, index, key, body } => {
                    let each_idx = self.eachs.len();
                    let list_expr = wrap_val(list);
                    let destructure = match index {
                        Some(ref idx_var) => format!("{{ item: {}, index: {} }}", item, idx_var),
                        None => format!("{{ item: {} }}", item),
                    };
                    let key_fn = match key {
                        Some(ref k) => format!("(item, i) => {}", wrap_val(k)),
                        None => "(item, i) => i".to_string(),
                    };
                    
                    self.eachs.push(EachInfo {
                        list: list.clone(), item: item.clone(), index: index.clone(),
                        key: key.clone(), idx: each_idx,
                        events: Vec::new(), binds: Vec::new(), toggles: Vec::new(), exprs: Vec::new(),
                    });
 
                    js.push_str(&format!(
                        "  const {} = document.createComment('zippy-each-{}');\n  {}.appendChild({});\n",
                        id, each_idx, parent_var, id
                    ));
                    
                    let prev_each = self.current_each;
                    self.current_each = Some(each_idx);
                    
                    let create_body_js = self.render_each_create_js(body, item, index.as_deref());
                    let init_body_js = self.render_each_init_js(&self.eachs[each_idx], item, index.as_deref());
                    
                    self.current_each = prev_each;
 
                    js.push_str(&format!(
                        "  let __eachDispose{0};\n  effect(() => {{\n    if (__eachDispose{0}) __eachDispose{0}();\n    __eachDispose{0} = reconcileEach({1}, {2}, {3}, ({4}) => {{\n      const __root = document.createDocumentFragment();\n{5}\n      return __root.firstElementChild || __root;\n    }}, (el, {4}) => {{\n      const __eachCleanup = [];\n      const on = (element, event, handler, cleanupArray) => {{\n        element.addEventListener(event, handler);\n        cleanupArray.push(() => element.removeEventListener(event, handler));\n      }};\n{6}\n      return () => {{ __eachCleanup.forEach(fn => fn()); }};\n    }});\n  }});\n  onDestroy(() => {{ if (__eachDispose{0}) __eachDispose{0}(); }});\n",
                        each_idx, id, list_expr, key_fn, destructure, create_body_js, init_body_js
                    ));
                }

            }
        }
        js
    }

    fn render_each_create_js(&mut self, nodes: &[Node], item_var: &str, index_var: Option<&str>) -> String {
        let mut js = String::new();
        let each_idx = self.current_each.unwrap();

        for (i, n) in nodes.iter().enumerate() {
            let id = format!("      _ei{}", i);
            match n {
                Node::Element { tag, attrs, children } => {
                    js.push_str(&format!("      const {} = document.createElement('{}');\n", id, tag));
                    for a in attrs {
                        match &a.value {
                            AttrValue::Static(v) => {
                                js.push_str(&format!("      {}.setAttribute('{}', '{}');\n", id, a.name, v));
                            }
                            AttrValue::Dynamic(e) => {
                                let idx = self.eachs[each_idx].exprs.len();
                                self.eachs[each_idx].exprs.push(e.clone());
                                js.push_str(&format!("      {}.setAttribute('data-zippy-expr-each-attr-{}', '{}');\n", id, idx, a.name));
                            }
                            AttrValue::Event(ev, handler) => {
                                let idx = self.eachs[each_idx].events.len();
                                self.eachs[each_idx].events.push((ev.clone(), handler.clone()));
                                js.push_str(&format!("      {}.setAttribute('data-zippy-evt-each-{}', '');\n", id, idx));
                            }
                            AttrValue::Bind(prop, expr) => {
                                let idx = self.eachs[each_idx].binds.len();
                                self.eachs[each_idx].binds.push((prop.clone(), expr.clone()));
                                js.push_str(&format!("      {}.setAttribute('data-zippy-bind-each-{}', '');\n", id, idx));
                            }
                            AttrValue::ClassToggle(class_name, expr) => {
                                let idx = self.eachs[each_idx].toggles.len();
                                self.eachs[each_idx].toggles.push((class_name.clone(), expr.clone()));
                                js.push_str(&format!("      {}.setAttribute('data-zippy-toggle-each-{}', '');\n", id, idx));
                            }
                        }
                    }
                    if !children.is_empty() {
                        js.push_str(&self.render_each_create_js(children, item_var, index_var));
                        // Child elements append logic
                        for (ci, _) in children.iter().enumerate() {
                            js.push_str(&format!("      {}.appendChild(_ei{});\n", id, ci));
                        }
                    }
                }
                Node::Text(t) => {
                    js.push_str(&format!("      const {} = document.createTextNode('{}');\n", id, t.replace('\'', "\\'")));
                }
                Node::Expr(e) => {
                    let idx = self.eachs[each_idx].exprs.len();
                    self.eachs[each_idx].exprs.push(e.clone());
                    js.push_str(&format!(
                        "      const {} = document.createElement('span');\n      {}.setAttribute('data-zippy-expr-each-{}', '');\n",
                        id, id, idx
                    ));
                }
                _ => {}
            }
        }
        js
    }

    fn render_each_init_js(&self, info: &EachInfo, item_var: &str, index_var: Option<&str>) -> String {
        let mut js = String::new();

        for (idx, (ev, handler)) in info.events.iter().enumerate() {
            js.push_str(&format!(
                "      const __btn{0} = el.querySelector('[data-zippy-evt-each-{0}]');\n      \
                 if (__btn{0}) on(__btn{0}, '{1}', {2}, __eachCleanup);\n",
                idx, ev, handler
            ));
        }

        for (idx, expr) in info.exprs.iter().enumerate() {
            let val = if expr == item_var || index_var.map(|v| v == expr).unwrap_or(false) {
                expr.clone()
            } else {
                wrap_val(expr)
            };

            js.push_str(&format!(
                "      const __attr{0} = el.querySelectorAll('[data-zippy-expr-each-attr-{0}]');\n      \
                 __attr{0}.forEach(__n => {{\n        \
                   const __name = __n.getAttribute('data-zippy-expr-each-attr-{0}');\n        \
                   __eachCleanup.push(effect(() => {{ __n.setAttribute(__name, {1}); }}));\n      \
                 }});\n",
                idx, val
            ));

            js.push_str(&format!(
                "      const __expr{0} = el.querySelector('[data-zippy-expr-each-{0}]');\n      \
                 if (__expr{0}) {{\n        \
                   __eachCleanup.push(effect(() => {{ __expr{0}.textContent = {1}; }}));\n      \
                 }}\n",
                idx, val
            ));
        }

        for (idx, (prop, expr)) in info.binds.iter().enumerate() {
            let val = wrap_val(expr);
            js.push_str(&format!(
                "      const __bind{0} = el.querySelector('[data-zippy-bind-each-{0}]');\n      \
                 if (__bind{0}) {{\n        \
                   __bind{0}.{1} = {2};\n        \
                   __bind{0}.addEventListener('input', () => {{ {3}.val = __bind{0}.{1}; }});\n        \
                   __eachCleanup.push(effect(() => {{ __bind{0}.{1} = {2}; }}));\n      \
                 }}\n",
                idx, prop, val, expr
            ));
        }

        for (idx, (class_name, expr)) in info.toggles.iter().enumerate() {
            let val = wrap_val(expr);
            js.push_str(&format!(
                "      const __toggle{0} = el.querySelector('[data-zippy-toggle-each-{0}]');\n      \
                 if (__toggle{0}) {{\n        \
                   __eachCleanup.push(effect(() => {{ __toggle{0}.classList.toggle('{1}', {2}); }}));\n      \
                 }}\n",
                idx, class_name, val
            ));
        }

        js
    }

    fn has_each(&self) -> bool { !self.eachs.is_empty() }
    fn has_events(&self) -> bool { self.events_count > 0 }

    fn render_unmount_comp(&self) -> String {
        if self.comps.is_empty() { return String::new(); }
        let mut code = String::new();
        for i in 0..self.comps.len() {
            code.push_str(&format!("\n    if (__cmp{}) __cmp{}.unmount();", i, i));
        }
        code
    }
}

#[allow(dead_code)]
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
        let (js, _) = generate(
            "let count = signal(0);",
            "<p>{count}</p>",
            "",
        ).unwrap();
        assert!(js.contains("signal"));
        assert!(js.contains("effect"));
        assert!(js.contains("count.val"));
        assert!(js.contains("export default function ZippyComponent"));
    }

    #[test]
    fn test_generate_bind() {
        let (js, _) = generate(
            "let x = signal('');",
            "<input bind:value={x} />",
            "",
        ).unwrap();
        assert!(js.contains("addEventListener('input'"));
        assert!(js.contains("x.val"));
    }

    #[test]
    fn test_generate_each_index() {
        let (js, _) = generate(
            "let items = signal([1,2,3]);",
            "{#each items as item, i}<li>{i}: {item}</li>{/each}",
            "",
        ).unwrap();
        assert!(js.contains("reconcileEach"));
    }

    #[test]
    fn test_generate_if() {
        let (js, _) = generate(
            "let show = signal(true);",
            "{#if show}<p>visible</p>{/if}",
            "",
        ).unwrap();
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
        let info = extract_imports("import Foo from './Foo.zippy'\nlet x = 1", ".js");
        assert_eq!(info.names, vec!["Foo"]);
        assert!(info.imports.contains("Foo"));
        assert!(info.imports.contains(".js"));
        assert!(info.body.contains("x = 1"));
    }
}
