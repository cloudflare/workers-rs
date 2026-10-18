//! Stopgap patches for generated `--experimental-generic-mono` bindings that do not compile.
//! Edits the source text at syn span locations so formatting and docs survive; each patched
//! item gets a `// ts-gen mono:` marker naming the issue.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use proc_macro2::LineColumn;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    FnArg, ForeignItem, GenericArgument, Generics, ImplItem, Item, PathArguments, Type,
    TypeParamBound,
};

const JS_GENERIC: &str = "::wasm_bindgen::JsGeneric";
const OPTION_ABI: &str = "::wasm_bindgen::convert::OptionIntoWasmAbi";
const VEC_REF_ABI: &str = "::wasm_bindgen::convert::VectorRefIntoWasmAbi";
const STATIC: &str = "'static";

struct Src {
    text: String,
    line_starts: Vec<usize>,
}

impl Src {
    fn new(text: String) -> Self {
        let mut line_starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i + 1);
            }
        }
        Src { text, line_starts }
    }

    fn offset(&self, lc: LineColumn) -> usize {
        let start = self.line_starts[lc.line - 1];
        let line = &self.text[start..];
        start
            + line
                .char_indices()
                .nth(lc.column)
                .map(|(i, _)| i)
                .unwrap_or(line.len())
    }

    fn line_start(&self, lc: LineColumn) -> usize {
        self.line_starts[lc.line - 1]
    }

    fn indent_at(&self, lc: LineColumn) -> String {
        let start = self.line_starts[lc.line - 1];
        self.text[start..]
            .chars()
            .take_while(|c| *c == ' ')
            .collect()
    }
}

#[derive(Default)]
struct FnArgIdents {
    depth: usize,
    out: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for FnArgIdents {
    fn visit_path_segment(&mut self, seg: &'ast syn::PathSegment) {
        if self.depth > 0 {
            self.out.insert(seg.ident.to_string());
        }
        let is_fn = seg.ident == "Function";
        if is_fn {
            self.depth += 1;
        }
        visit::visit_path_segment(self, seg);
        if is_fn {
            self.depth -= 1;
        }
    }
}

fn single_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => {
            let seg = &p.path.segments[0];
            matches!(seg.arguments, PathArguments::None).then(|| seg.ident.to_string())
        }
        _ => None,
    }
}

/// `Option<P>` and `&[P]` argument shapes over a bare type parameter.
fn abi_needs(ty: &Type, needs: &mut BTreeMap<String, BTreeSet<&'static str>>) {
    match ty {
        Type::Path(p) => {
            if let Some(seg) = p.path.segments.last() {
                if seg.ident == "Option" {
                    if let PathArguments::AngleBracketed(a) = &seg.arguments {
                        if let Some(GenericArgument::Type(t)) = a.args.first() {
                            if let Some(id) = single_ident(t) {
                                needs.entry(id).or_default().insert(OPTION_ABI);
                            }
                        }
                    }
                }
            }
        }
        Type::Reference(r) => {
            if let Type::Slice(s) = &*r.elem {
                if let Some(id) = single_ident(&s.elem) {
                    needs.entry(id).or_default().insert(VEC_REF_ABI);
                }
            }
        }
        _ => {}
    }
}

fn type_params(g: &Generics) -> BTreeSet<String> {
    g.type_params().map(|t| t.ident.to_string()).collect()
}

fn has_bound(tp: &syn::TypeParam, bound: &str) -> bool {
    let last = bound.rsplit("::").next().unwrap();
    tp.bounds.iter().any(|b| match b {
        TypeParamBound::Trait(t) => t.path.segments.last().is_some_and(|s| s.ident == last),
        TypeParamBound::Lifetime(l) => format!("'{}", l.ident) == bound,
        _ => false,
    })
}

struct Patcher<'a> {
    src: &'a Src,
    inserts: Vec<(usize, String)>,
    deletes: Vec<(usize, usize, String)>,
    marked: BTreeSet<usize>,
    counts: BTreeMap<&'static str, usize>,
}

impl Patcher<'_> {
    fn mark(&mut self, at: LineColumn, why: &str) {
        let off = self.src.line_start(at);
        if self.marked.insert(off) {
            let indent = self.src.indent_at(at);
            self.inserts
                .push((off, format!("{indent}// ts-gen mono: {why}\n")));
        }
    }

    fn bound(&mut self, g: &Generics, needs: &BTreeMap<String, BTreeSet<&'static str>>) -> bool {
        let mut any = false;
        for tp in g.type_params() {
            let Some(want) = needs.get(&tp.ident.to_string()) else {
                continue;
            };
            let missing: Vec<&str> = want.iter().copied().filter(|b| !has_bound(tp, b)).collect();
            if missing.is_empty() {
                continue;
            }
            any = true;
            let joined = missing.join(" + ");
            if let Some(last) = tp.bounds.last() {
                let off = self.src.offset(last.span().end());
                self.inserts.push((off, format!(" + {joined}")));
            } else {
                let off = self.src.offset(tp.ident.span().end());
                self.inserts.push((off, format!(": {joined}")));
            }
        }
        any
    }
}

fn first_span<T: Spanned>(attrs: &[syn::Attribute], fallback: &T) -> LineColumn {
    attrs
        .first()
        .map(|a| a.span().start())
        .unwrap_or_else(|| fallback.span().start())
}

fn impl_base(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

fn walk(items: &[Item], path: &str, out: &mut Vec<(String, Item)>) {
    for item in items {
        if let Item::Mod(m) = item {
            if let Some((_, inner)) = &m.content {
                walk(inner, &format!("{path}::{}", m.ident), out);
            }
        } else {
            out.push((path.to_string(), item.clone()));
        }
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("bindings.rs");
    let src = Src::new(std::fs::read_to_string(&path).unwrap());
    let file = syn::parse_file(&src.text).unwrap();
    let mut items = vec![];
    walk(&file.items, "", &mut items);

    // Declared arity of every extern type, for twins that name an erased parameter.
    let mut arity: HashMap<(String, String), usize> = HashMap::new();
    for (m, item) in &items {
        if let Item::ForeignMod(fm) = item {
            for fi in &fm.items {
                if let ForeignItem::Type(t) = fi {
                    arity.insert((m.clone(), t.ident.to_string()), t.generics.params.len());
                }
            }
        }
    }

    // Bounds an inherent impl's parameters need, unified over `X` and `XBuilder` because the
    // factories on `X` call into the builder.
    let mut impl_needs: HashMap<(String, String), BTreeMap<String, BTreeSet<&'static str>>> =
        HashMap::new();
    for (m, item) in &items {
        let Item::Impl(i) = item else { continue };
        if i.trait_.is_some() {
            continue;
        }
        let Some(base) = impl_base(&i.self_ty) else {
            continue;
        };
        let key = (
            m.clone(),
            base.strip_suffix("Builder").unwrap_or(&base).to_string(),
        );
        let params = type_params(&i.generics);
        let entry = impl_needs.entry(key).or_default();
        for ii in &i.items {
            let ImplItem::Fn(f) = ii else { continue };
            let mut v = FnArgIdents::default();
            v.visit_signature(&f.sig);
            for p in v.out.intersection(&params) {
                entry.entry(p.clone()).or_default().insert(JS_GENERIC);
            }
            let mut abi = BTreeMap::new();
            for input in &f.sig.inputs {
                if let FnArg::Typed(pt) = input {
                    abi_needs(&pt.ty, &mut abi);
                }
            }
            for (p, b) in abi {
                if params.contains(&p) {
                    entry.entry(p).or_default().extend(b);
                }
            }
        }
    }

    let mut p = Patcher {
        src: &src,
        inserts: vec![],
        deletes: vec![],
        marked: BTreeSet::new(),
        counts: BTreeMap::new(),
    };

    for (m, item) in &items {
        match item {
            Item::ForeignMod(fm) => {
                for fi in &fm.items {
                    let ForeignItem::Fn(f) = fi else { continue };
                    let params = type_params(&f.sig.generics);
                    let mut v = FnArgIdents::default();
                    v.visit_signature(&f.sig);
                    let needs: BTreeMap<_, _> = v
                        .out
                        .intersection(&params)
                        .map(|p| (p.clone(), BTreeSet::from([JS_GENERIC])))
                        .collect();
                    if p.bound(&f.sig.generics, &needs) {
                        p.mark(
                            first_span(&f.attrs, &f.sig),
                            "`JsGeneric` on type parameters inside `Function<…>` (generic-mono.md #1)",
                        );
                        *p.counts.entry("#1 extern fn bounds").or_default() += 1;
                    }

                    // A `_js_string` twin that passes more arguments than the type declares.
                    let name = f.sig.ident.to_string();
                    if name.contains("_js_string") {
                        let mut over = false;
                        let mut stack: Vec<&Type> = vec![];
                        if let syn::ReturnType::Type(_, t) = &f.sig.output {
                            stack.push(t);
                        }
                        struct Over<'a> {
                            m: &'a str,
                            arity: &'a HashMap<(String, String), usize>,
                            hit: bool,
                        }
                        impl<'ast> Visit<'ast> for Over<'_> {
                            fn visit_path_segment(&mut self, seg: &'ast syn::PathSegment) {
                                if let PathArguments::AngleBracketed(a) = &seg.arguments {
                                    if let Some(n) =
                                        self.arity.get(&(self.m.to_string(), seg.ident.to_string()))
                                    {
                                        if a.args.len() > *n {
                                            self.hit = true;
                                        }
                                    }
                                }
                                visit::visit_path_segment(self, seg);
                            }
                        }
                        let mut o = Over {
                            m,
                            arity: &arity,
                            hit: false,
                        };
                        o.visit_signature(&f.sig);
                        over |= o.hit;
                        if over {
                            let start = src.line_start(first_span(&f.attrs, &f.sig));
                            let end = src.offset(f.semi_token.span.end()) + 1;
                            let indent = src.indent_at(first_span(&f.attrs, &f.sig));
                            p.deletes.push((
                                start,
                                end,
                                format!(
                                    "{indent}// ts-gen mono: `{name}` removed; its return passes an \
                                     argument for an erased type parameter (generic-mono.md #3)\n"
                                ),
                            ));
                            *p.counts.entry("#3 removed twins").or_default() += 1;
                        }
                    }
                }
            }
            Item::Impl(i) if i.trait_.is_none() => {
                let Some(base) = impl_base(&i.self_ty) else {
                    continue;
                };
                let key = (
                    m.clone(),
                    base.strip_suffix("Builder").unwrap_or(&base).to_string(),
                );
                if let Some(needs) = impl_needs.get(&key) {
                    if p.bound(&i.generics, needs) {
                        p.mark(
                            first_span(&i.attrs, &i.impl_token),
                            "bounds the generated methods need (generic-mono.md #1, #2)",
                        );
                        *p.counts.entry("#1/#2 impl bounds").or_default() += 1;
                    }
                }
                let mut seen: HashMap<String, usize> = HashMap::new();
                for ii in &i.items {
                    let ImplItem::Fn(f) = ii else { continue };
                    let name = f.sig.ident.to_string();
                    let n = seen.entry(name.clone()).or_default();
                    *n += 1;
                    let body = quote::quote!(#f).to_string();
                    if *n > 1 && body.contains("_js_string_js") {
                        let off = src.offset(f.sig.ident.span().end());
                        p.inserts.push((off, "_js_string".into()));
                        p.mark(
                            first_span(&f.attrs, &f.sig),
                            "`_js_string` twin of an inline wrapper took the base name (generic-mono.md #4)",
                        );
                        *p.counts.entry("#4 renamed twins").or_default() += 1;
                    }
                    let own: BTreeMap<_, _> = f
                        .sig
                        .generics
                        .type_params()
                        .filter(|t| t.bounds.is_empty())
                        .map(|t| (t.ident.to_string(), BTreeSet::from([STATIC])))
                        .collect();
                    if !own.is_empty() && body.contains("_js (") && p.bound(&f.sig.generics, &own) {
                        p.mark(
                            first_span(&f.attrs, &f.sig),
                            "`'static` on the inline wrapper's type parameters (generic-mono.md #5)",
                        );
                        *p.counts.entry("#5 'static bounds").or_default() += 1;
                    }
                }
            }
            _ => {}
        }
    }

    let mut text = src.text.clone();
    let mut edits: Vec<(usize, usize, String)> = p
        .inserts
        .into_iter()
        .map(|(o, s)| (o, o, s))
        .chain(p.deletes)
        .collect();
    edits.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    for (start, end, s) in edits {
        text.replace_range(start..end, &s);
    }
    std::fs::write(&path, text).unwrap();
    for (k, v) in p.counts {
        eprintln!("{k}: {v}");
    }
}
