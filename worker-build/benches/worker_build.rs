use divan::Bencher;

fn main() {
    divan::main();
}

/// Sample Cargo.lock content representative of a typical workers-rs project.
const SAMPLE_LOCKFILE: &str = r#"
version = 3

[[package]]
name = "my-worker"
version = "0.1.0"
dependencies = [
 "serde 1.0.164",
 "wasm-bindgen 0.2.112",
 "worker 0.7.4",
]

[[package]]
name = "serde"
version = "1.0.164"

[[package]]
name = "serde_json"
version = "1.0.140"

[[package]]
name = "wasm-bindgen"
version = "0.2.112"

[[package]]
name = "worker"
version = "0.7.4"
dependencies = [
 "serde 1.0.164",
 "wasm-bindgen 0.2.112",
]

[[package]]
name = "js-sys"
version = "0.3.83"

[[package]]
name = "web-sys"
version = "0.3.89"

[[package]]
name = "futures-util"
version = "0.3.31"

[[package]]
name = "http"
version = "1.3.0"

[[package]]
name = "url"
version = "2.4.0"

[[package]]
name = "chrono"
version = "0.4.41"

[[package]]
name = "tokio"
version = "1.28.0"

[[package]]
name = "worker-macros"
version = "0.7.4"

[[package]]
name = "worker-sys"
version = "0.7.4"
"#;

/// Sample Cargo.toml manifest content.
const SAMPLE_MANIFEST: &str = r#"
[package]
name = "my-worker"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
worker = { version = "0.7.4", features = ["queue", "d1"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-O"]

[package.metadata.wasm-pack.profile.release.wasm-bindgen]
debug-js-glue = false
demangle-name-section = true
dwarf-debug-info = false
omit-default-module-path = false
"#;

/// Sample wasm-bindgen generated JavaScript output for handler extraction.
const SAMPLE_JS_OUTPUT: &str = r#"
import { __wbg_set_wasm } from './index_bg.js';
import * as wasm from './index_bg.wasm';
__wbg_set_wasm(wasm);
export function fetch(req, env, ctx) {
    return wasm.fetch(req, env, ctx);
}
export function scheduled(event, env, ctx) {
    return wasm.scheduled(event, env, ctx);
}
export function queue(batch, env, ctx) {
    return wasm.queue(batch, env, ctx);
}
export function __wbg_reset_state() {
    return wasm.__wbg_reset_state();
}
export function setPanicHook() {
    wasm.setPanicHook();
}
export class MyDurableObject {
    constructor(state, env) {
        this.state = state;
        this.env = env;
    }
}
export class AnotherDurable {
    constructor(state, env) {
        this.state = state;
    }
}
export { SomeHelper as helper_func };
"#;

mod lockfile_benchmarks {
    use super::*;

    #[divan::bench]
    fn parse_lockfile_toml(bencher: Bencher) {
        bencher.bench(|| toml::from_str::<toml::Value>(SAMPLE_LOCKFILE).unwrap());
    }

    #[divan::bench(args = [5, 25, 100])]
    fn parse_lockfile_with_n_packages(bencher: Bencher, n: usize) {
        let lockfile = generate_lockfile(n);
        bencher.bench(|| toml::from_str::<toml::Value>(&lockfile).unwrap());
    }

    fn generate_lockfile(n: usize) -> String {
        let mut content = String::from("version = 3\n\n");
        content
            .push_str("[[package]]\nname = \"my-worker\"\nversion = \"0.1.0\"\ndependencies = [\n");
        for i in 0..n {
            content.push_str(&format!(" \"dep-{i} 1.0.{i}\",\n"));
        }
        content.push_str("]\n\n");
        for i in 0..n {
            content.push_str(&format!(
                "[[package]]\nname = \"dep-{i}\"\nversion = \"1.0.{i}\"\n\n"
            ));
        }
        content
    }
}

mod manifest_benchmarks {
    use super::*;

    #[divan::bench]
    fn parse_manifest_toml(bencher: Bencher) {
        bencher.bench(|| toml::from_str::<toml::Value>(SAMPLE_MANIFEST).unwrap());
    }

    #[divan::bench]
    fn parse_manifest_with_serde_ignored(bencher: Bencher) {
        bencher.bench(|| {
            let deserializer = toml::Deserializer::parse(SAMPLE_MANIFEST).unwrap();
            let _: toml::Value = serde_ignored::deserialize(deserializer, |_path| {}).unwrap();
        });
    }
}

mod handler_benchmarks {
    use super::*;

    /// Benchmark extracting function exports from wasm-bindgen generated JS,
    /// mirroring the logic in generate_handlers.
    #[divan::bench]
    fn extract_function_exports(bencher: Bencher) {
        let content = SAMPLE_JS_OUTPUT;
        let system_fns: &[&str] = &["__wbg_reset_state", "setPanicHook"];
        bencher.bench(|| {
            let mut func_names = Vec::new();
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("export function") {
                    if let Some(bracket_pos) = rest.find('(') {
                        let func_name = rest[..bracket_pos].trim();
                        if !system_fns.contains(&func_name) {
                            func_names.push(func_name);
                        }
                    }
                } else if let Some(rest) = line.strip_prefix("export {") {
                    if let Some(as_pos) = rest.find(" as ") {
                        let rest = &rest[as_pos + 4..];
                        if let Some(brace_pos) = rest.find('}') {
                            let func_name = rest[..brace_pos].trim();
                            if !system_fns.contains(&func_name) {
                                func_names.push(func_name);
                            }
                        }
                    }
                }
            }
            func_names
        });
    }

    /// Benchmark extracting class exports from wasm-bindgen generated JS,
    /// mirroring the logic in add_export_wrappers.
    #[divan::bench]
    fn extract_class_exports(bencher: Bencher) {
        let content = SAMPLE_JS_OUTPUT;
        bencher.bench(|| {
            let mut class_names = Vec::new();
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("export class ") {
                    if let Some(brace_pos) = rest.find('{') {
                        let class_name = rest[..brace_pos].trim();
                        class_names.push(class_name.to_string());
                    }
                }
            }
            class_names
        });
    }

    /// Benchmark generating handler wrapper code, mirroring the logic
    /// that produces Entrypoint.prototype assignments.
    #[divan::bench]
    fn generate_handler_wrappers(bencher: Bencher) {
        let func_names = vec!["fetch", "scheduled", "queue", "custom_handler"];
        bencher.bench(|| {
            let mut handlers = String::new();
            for func_name in &func_names {
                if *func_name == "fetch" || *func_name == "queue" || *func_name == "scheduled" {
                    handlers += &format!(
                        "Entrypoint.prototype.{func_name} = function {func_name} (arg) {{\n  return exports.{func_name}.call(this, arg, this.env, this.ctx);\n}}\n"
                    );
                } else {
                    handlers +=
                        &format!("Entrypoint.prototype.{func_name} = exports.{func_name};\n");
                }
            }
            handlers
        });
    }

    #[divan::bench(args = [5, 20, 50])]
    fn generate_handler_wrappers_n_handlers(bencher: Bencher, n: usize) {
        let func_names: Vec<String> = (0..n).map(|i| format!("handler_{i}")).collect();
        bencher.bench(|| {
            let mut handlers = String::new();
            for func_name in &func_names {
                handlers += &format!("Entrypoint.prototype.{func_name} = exports.{func_name};\n");
            }
            handlers
        });
    }
}

mod version_benchmarks {
    use super::*;

    #[divan::bench]
    fn parse_semver_versions(bencher: Bencher) {
        let versions = ["0.2.112", "1.0.164", "0.7.4", "0.3.83", "0.3.89", "0.4.41"];
        bencher.bench(|| {
            versions
                .iter()
                .map(|v| semver::Version::parse(v).unwrap())
                .collect::<Vec<_>>()
        });
    }

    #[divan::bench]
    fn semver_version_req_matching(bencher: Bencher) {
        let req = semver::VersionReq::parse("^0.2.106").unwrap();
        let versions: Vec<semver::Version> = vec![
            semver::Version::parse("0.2.112").unwrap(),
            semver::Version::parse("0.2.106").unwrap(),
            semver::Version::parse("0.2.100").unwrap(),
            semver::Version::parse("0.3.0").unwrap(),
        ];
        bencher.bench(|| versions.iter().map(|v| req.matches(v)).collect::<Vec<_>>());
    }

    #[divan::bench]
    fn levenshtein_distance(bencher: Bencher) {
        let target = "package.metadata.wasm-pack";
        let inputs = [
            "package.metadata.wasm-pack",
            "package.metadata.wasm-pak",
            "package.metadata.wasmpack",
            "package.metadata.wasm_pack",
            "package.metadata.other-tool",
        ];
        bencher.bench(|| {
            inputs
                .iter()
                .map(|input| strsim::levenshtein(target, input))
                .collect::<Vec<_>>()
        });
    }
}
