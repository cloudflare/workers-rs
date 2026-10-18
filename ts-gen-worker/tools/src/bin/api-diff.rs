//! Signature-level diff of two generated bindings files.
//!
//! Every public item is keyed by (module, owner type, item) and rendered as a one-line
//! signature without docs, bodies or wasm-bindgen marshalling attributes; the diff lists every
//! key whose rendering differs.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use quote::ToTokens;
use syn::{Attribute, FnArg, ForeignItem, ImplItem, Item, ReturnType, Type};

type Api = BTreeMap<(String, String), BTreeMap<String, String>>;

fn collapse(s: &str) -> String {
    let mut out = s.split_whitespace().collect::<Vec<_>>().join(" ");
    for (from, to) in [
        ("( ", "("),
        (", )", ")"),
        (" )", ")"),
        ("< ", "<"),
        (", >", ">"),
        (", }", " }"),
    ] {
        while out.contains(from) {
            out = out.replace(from, to);
        }
    }
    out
}

fn pretty_file(items: Vec<Item>) -> String {
    let file = syn::File {
        shebang: None,
        attrs: vec![],
        items,
    };
    prettyplease::unparse(&file)
}

fn render_item(item: Item) -> String {
    collapse(pretty_file(vec![item]).trim())
}

fn render_foreign(item: ForeignItem) -> String {
    let wrapped: Item = syn::parse_quote!(extern "C" { #item });
    let s = render_item(wrapped);
    let s = s.trim_start_matches("extern \"C\" {").trim_end_matches('}');
    s.trim().to_string()
}

fn tokens(t: impl ToTokens) -> String {
    collapse(
        &t.to_token_stream()
            .to_string()
            .replace(" :: ", "::")
            .replace(":: ", "::")
            .replace(" ,", ",")
            .replace(" < ", "<")
            .replace(" <", "<")
            .replace("< ", "<")
            .replace(" >", ">")
            .replace("& ", "&"),
    )
}

fn base_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Reference(r) => base_ident(&r.elem),
        Type::Path(p) => {
            let seg = p.path.segments.last()?;
            if (seg.ident == "Result" || seg.ident == "Option") && p.path.segments.len() == 1 {
                if let syn::PathArguments::AngleBracketed(a) = &seg.arguments {
                    if let Some(syn::GenericArgument::Type(t)) = a.args.first() {
                        return base_ident(t);
                    }
                }
            }
            Some(seg.ident.to_string())
        }
        _ => None,
    }
}

fn wb_args(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("wasm_bindgen"))
        .filter_map(|a| match &a.meta {
            syn::Meta::List(l) => Some(tokens(&l.tokens)),
            _ => None,
        })
        .collect()
}

fn derives(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
        .filter_map(|a| match &a.meta {
            syn::Meta::List(l) => Some(tokens(&l.tokens)),
            _ => None,
        })
        .collect()
}

fn split_top(s: &str) -> Vec<String> {
    let mut out = vec![];
    let mut depth = 0i32;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '(' | '<' | '[' => depth += 1,
            ')' | '>' | ']' => depth -= 1,
            ',' if depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

fn strip_attrs(attrs: &mut Vec<Attribute>) {
    attrs.clear();
}

struct Collector {
    api: Api,
}

impl Collector {
    fn add(&mut self, module: &str, owner: &str, key: String, sig: String) {
        let entry = self
            .api
            .entry((module.to_string(), owner.to_string()))
            .or_default();
        let mut k = key.clone();
        let mut n = 2;
        while entry.contains_key(&k) {
            k = format!("{key} #{n}");
            n += 1;
        }
        entry.insert(k, sig);
    }

    fn items(&mut self, module: &str, items: &[Item]) {
        for item in items {
            self.item(module, item);
        }
    }

    fn item(&mut self, module: &str, item: &Item) {
        match item {
            Item::ForeignMod(m) => {
                for fi in &m.items {
                    self.foreign(module, fi);
                }
            }
            Item::Impl(i) => {
                let owner = base_ident(&i.self_ty).unwrap_or_else(|| tokens(&i.self_ty));
                let mut header = i.clone();
                header.items.clear();
                strip_attrs(&mut header.attrs);
                let header_s = render_item(Item::Impl(header))
                    .trim_end_matches("{}")
                    .trim()
                    .to_string();
                if let Some((_, path, _)) = &i.trait_ {
                    self.add(module, &owner, format!("impl {}", tokens(path)), header_s);
                    return;
                }
                self.add(
                    module,
                    &owner,
                    format!("impl {}", tokens(&i.self_ty)),
                    header_s,
                );
                for ii in &i.items {
                    match ii {
                        ImplItem::Fn(f) => {
                            let mut f = f.clone();
                            strip_attrs(&mut f.attrs);
                            let s = render_sig(f.vis.clone(), f.sig.clone());
                            self.add(module, &owner, format!("fn {}", f.sig.ident), s);
                        }
                        ImplItem::Const(c) => {
                            let mut c = c.clone();
                            strip_attrs(&mut c.attrs);
                            self.add(
                                module,
                                &owner,
                                format!("const {}", c.ident),
                                tokens(quote::quote!(#c)),
                            );
                        }
                        other => {
                            self.add(module, &owner, "impl item".into(), tokens(other));
                        }
                    }
                }
            }
            Item::Struct(s) => {
                let mut s2 = s.clone();
                let ds = derives(&s.attrs);
                strip_attrs(&mut s2.attrs);
                let mut sig = render_item(Item::Struct(s2));
                if !ds.is_empty() {
                    sig = format!("{sig} derive({})", ds.join(", "));
                }
                self.add(module, &s.ident.to_string(), "struct".into(), sig);
            }
            Item::Enum(e) => {
                let owner = e.ident.to_string();
                let mut header = e.clone();
                header.variants.clear();
                let ds = derives(&e.attrs);
                let wb = wb_args(&e.attrs);
                strip_attrs(&mut header.attrs);
                let mut sig = render_item(Item::Enum(header));
                if !ds.is_empty() {
                    sig = format!("{sig} derive({})", ds.join(", "));
                }
                if !wb.is_empty() {
                    sig = format!("{sig} wasm_bindgen({})", wb.join("; "));
                }
                self.add(module, &owner, "enum".into(), sig);
                for v in &e.variants {
                    let mut v = v.clone();
                    strip_attrs(&mut v.attrs);
                    self.add(module, &owner, format!("variant {}", v.ident), tokens(&v));
                }
            }
            Item::Type(t) => {
                let mut t2 = t.clone();
                strip_attrs(&mut t2.attrs);
                self.add(
                    module,
                    &t.ident.to_string(),
                    "type alias".into(),
                    render_item(Item::Type(t2)),
                );
            }
            Item::Mod(m) => {
                if let Some((_, items)) = &m.content {
                    let path = if module.is_empty() {
                        m.ident.to_string()
                    } else {
                        format!("{module}::{}", m.ident)
                    };
                    self.items(&path, items);
                }
            }
            Item::Fn(f) => {
                let s = render_sig(f.vis.clone(), f.sig.clone());
                self.add(module, "(free functions)", format!("fn {}", f.sig.ident), s);
            }
            Item::Use(_) | Item::Macro(_) => {}
            other => {
                let mut o = other.clone();
                if let Item::Const(c) = &mut o {
                    strip_attrs(&mut c.attrs);
                }
                if let Item::Static(s) = &mut o {
                    strip_attrs(&mut s.attrs);
                }
                self.add(module, "(other)", "item".into(), render_item(o));
            }
        }
    }

    fn foreign(&mut self, module: &str, fi: &ForeignItem) {
        match fi {
            ForeignItem::Type(t) => {
                let wb = wb_args(&t.attrs);
                let extends: Vec<String> = wb
                    .iter()
                    .flat_map(|a| split_top(a))
                    .filter_map(|a| a.strip_prefix("extends = ").map(str::to_string))
                    .collect();
                let ds = derives(&t.attrs);
                let mut t2 = t.clone();
                strip_attrs(&mut t2.attrs);
                let mut sig = render_foreign(ForeignItem::Type(t2));
                if !extends.is_empty() {
                    sig = format!("{sig} extends({})", extends.join(", "));
                }
                if !ds.is_empty() {
                    sig = format!("{sig} derive({})", ds.join(", "));
                }
                self.add(module, &t.ident.to_string(), "type".into(), sig);
            }
            ForeignItem::Fn(f) => {
                let wb = wb_args(&f.attrs);
                let flags: Vec<String> = wb.iter().flat_map(|a| split_top(a)).collect();
                let has = |n: &str| flags.iter().any(|f| f == n);
                let static_of = flags
                    .iter()
                    .find_map(|f| f.strip_prefix("static_method_of = ").map(str::to_string));
                let owner = if has("method") {
                    match f.sig.inputs.first() {
                        Some(FnArg::Typed(p)) => base_ident(&p.ty),
                        _ => None,
                    }
                } else if let Some(s) = static_of {
                    Some(s.rsplit("::").next().unwrap().to_string())
                } else if has("constructor") {
                    match &f.sig.output {
                        ReturnType::Type(_, t) => base_ident(t),
                        _ => None,
                    }
                } else {
                    None
                }
                .unwrap_or_else(|| "(free functions)".into());
                let mut f2 = f.clone();
                strip_attrs(&mut f2.attrs);
                for input in f2.sig.inputs.iter_mut() {
                    if let FnArg::Typed(pt) = input {
                        pt.attrs.clear();
                    }
                }
                let mut sig = render_foreign(ForeignItem::Fn(f2));
                if has("variadic") {
                    sig = format!("{sig} [variadic]");
                }
                self.add(module, &owner, format!("fn {}", f.sig.ident), sig);
            }
            ForeignItem::Static(s) => {
                let mut s2 = s.clone();
                strip_attrs(&mut s2.attrs);
                self.add(
                    module,
                    "(statics)",
                    format!("static {}", s.ident),
                    render_foreign(ForeignItem::Static(s2)),
                );
            }
            other => {
                self.add(module, "(other)", "foreign".into(), tokens(other));
            }
        }
    }
}

fn render_sig(vis: syn::Visibility, sig: syn::Signature) -> String {
    render_foreign(ForeignItem::Fn(syn::ForeignItemFn {
        attrs: vec![],
        vis,
        sig,
        semi_token: Default::default(),
    }))
}

fn load(path: &str) -> Api {
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let file = syn::parse_file(&src).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut c = Collector {
        api: BTreeMap::new(),
    };
    c.items("", &file.items);
    c.api
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (old_path, new_path) = (&args[1], &args[2]);
    let title = args.get(3).cloned().unwrap_or_default();
    let old = load(old_path);
    let new = load(new_path);

    let mut owners: Vec<&(String, String)> = old.keys().chain(new.keys()).collect();
    owners.sort();
    owners.dedup();

    let empty = BTreeMap::new();
    let (mut removed, mut added, mut changed, mut same, mut owners_changed) = (0, 0, 0, 0, 0);
    let mut body = String::new();
    for owner in owners {
        let o = old.get(owner).unwrap_or(&empty);
        let n = new.get(owner).unwrap_or(&empty);
        let mut keys: Vec<&String> = o.keys().chain(n.keys()).collect();
        keys.sort();
        keys.dedup();
        let mut lines = vec![];
        for k in keys {
            match (o.get(k), n.get(k)) {
                (Some(a), Some(b)) if a == b => same += 1,
                (Some(a), Some(b)) => {
                    changed += 1;
                    lines.push(format!("- `{a}` → `{b}`"));
                }
                (Some(a), None) => {
                    removed += 1;
                    lines.push(format!("- `{a}` → removed"));
                }
                (None, Some(b)) => {
                    added += 1;
                    lines.push(format!("- added → `{b}`"));
                }
                (None, None) => unreachable!(),
            }
        }
        if lines.is_empty() {
            continue;
        }
        owners_changed += 1;
        let name = if owner.0.is_empty() {
            owner.1.clone()
        } else {
            format!("{}::{}", owner.0, owner.1)
        };
        let status = match (old.contains_key(owner), new.contains_key(owner)) {
            (true, false) => " (owner removed)",
            (false, true) => " (owner added)",
            _ => "",
        };
        writeln!(body, "\n### `{name}`{status}\n").unwrap();
        for l in lines {
            writeln!(body, "{l}").unwrap();
        }
    }
    println!("{title}");
    println!(
        "{owners_changed} owners touched; {changed} items changed, {removed} removed, {added} added, \
         {same} unchanged."
    );
    print!("{body}");
}
