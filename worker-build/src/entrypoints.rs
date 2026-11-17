use anyhow::{bail, Result};

const PREFIX: &str = "__worker_entrypoint__";
const FETCH_PREFIX: &str = "__worker_entrypoint__fetch__";

pub fn is_named_handler(name: &str) -> bool {
    name.starts_with(PREFIX)
}

pub fn function_exports(source: &str) -> Vec<&str> {
    crate::export_decls(source)
        .filter_map(|line| {
            if let Some(rest) = line
                .strip_prefix("export function ")
                .or_else(|| line.strip_prefix("export async function "))
            {
                return rest.split_once('(').map(|(name, _)| name.trim());
            }
            let rest = line.strip_prefix("export {")?;
            let (_, alias) = rest.split_once(" as ")?;
            alias.split_once('}').map(|(name, _)| name.trim())
        })
        .collect()
}

pub fn class_exports(source: &str) -> Vec<&str> {
    crate::export_decls(source)
        .filter_map(|line| {
            if let Some(rest) = line.strip_prefix("export class ") {
                return rest.split_once('{').map(|(name, _)| name.trim());
            }
            let (name, definition) = line.strip_prefix("export var ")?.split_once('=')?;
            definition
                .trim_start()
                .starts_with("class")
                .then(|| name.trim())
        })
        .collect()
}

pub struct NamedEntrypoints<'a> {
    handlers: Vec<(&'a str, &'a str)>,
}

impl<'a> NamedEntrypoints<'a> {
    pub fn parse(source: &'a str) -> Result<Self> {
        let functions = function_exports(source);
        let classes = class_exports(source);
        let mut handlers = Vec::new();
        for function in &functions {
            if !is_named_handler(function) {
                continue;
            }
            let Some(name) = function.strip_prefix(FETCH_PREFIX) else {
                bail!("Unsupported named entrypoint handler: {function}");
            };
            let mut characters = name.chars();
            let valid = characters
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
                && characters
                    .all(|character| character.is_ascii_alphanumeric() || character == '_');
            if !valid || name == "default" {
                bail!("Invalid named entrypoint: {name:?}");
            }
            if functions.contains(&name) || classes.contains(&name) || name == "wasmModule" {
                bail!("Named entrypoint {name:?} conflicts with another export");
            }
            if handlers.iter().any(|(existing, _)| *existing == name) {
                bail!("Duplicate named entrypoint: {name}");
            }
            handlers.push((name, *function));
        }
        Ok(Self { handlers })
    }

    pub fn generate(&self, imports: &str, proxy: bool, run_to_completion: bool) -> String {
        let mut output = String::new();
        for (index, (name, function)) in self.handlers.iter().enumerate() {
            let binding = format!("__worker_entrypoint_{index}");
            let class = format!(
                "class extends WorkerEntrypoint {{
  async fetch(request) {{
    const response = {imports}.{function}(request, this.env, this.ctx);
    {}return await response;
  }}
}}",
                if run_to_completion {
                    "this.ctx.waitUntil(response);\n    "
                } else {
                    ""
                }
            );
            let value = if proxy {
                format!("new Proxy({class}, classProxyHooks)")
            } else {
                class
            };
            output.push_str(&format!(
                "const {binding} = {value};\nexport {{ {binding} as {name} }};\n"
            ));
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_named_handlers_and_preserves_export_names() {
        let source = "export function fetch() {}\nexport function __worker_entrypoint__fetch__First() {}\nexport { other as __worker_entrypoint__fetch__Second };";
        let entrypoints = NamedEntrypoints::parse(source).unwrap();
        assert_eq!(
            entrypoints.handlers,
            [
                ("First", "__worker_entrypoint__fetch__First"),
                ("Second", "__worker_entrypoint__fetch__Second"),
            ]
        );
        let default_handlers: Vec<_> = function_exports(source)
            .into_iter()
            .filter(|name| !is_named_handler(name))
            .collect();
        assert_eq!(default_handlers, ["fetch"]);
    }

    #[test]
    fn recognizes_emscripten_exports() {
        let source = "  export async function __worker_entrypoint__fetch__Loopback(){};export var Counter = class Counter{};";
        assert_eq!(
            NamedEntrypoints::parse(source).unwrap().handlers,
            [("Loopback", "__worker_entrypoint__fetch__Loopback")],
        );
        assert_eq!(class_exports(source), ["Counter"]);
    }

    #[test]
    fn rejects_conflicting_exports() {
        for export in [
            "export class Loopback {}",
            "export var Loopback = class Loopback {};",
            "export function Loopback() {}",
            "export function __worker_entrypoint__fetch__Loopback() {}",
        ] {
            let source =
                format!("export function __worker_entrypoint__fetch__Loopback() {{}}\n{export}");
            assert!(NamedEntrypoints::parse(&source).is_err());
        }
    }
}
