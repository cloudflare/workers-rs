use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Result;
use swc::{
    common::{FileName, Globals, SourceMap},
    ecmascript::ast::{EsVersion, KeyValueProp},
};
use swc_bundler::{Bundler, Config, Hook, Load, ModuleData, Resolve};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

pub fn bundle_worker(shim_src: String, mut dst: impl Write) -> Result<()> {
    let globals = Globals::default();
    let cm = Arc::new(SourceMap::default());

    let mut bundler = Bundler::new(
        &globals,
        cm.clone(),
        BundleLoader(cm.clone()),
        BundleResolver(Path::new("./build/worker").into()),
        Config {
            // In the future it could be nice for us to allow to have snippets that need wasm
            // probably blocked on https://github.com/rustwasm/wasm-bindgen/issues/2375.
            external_modules: vec!["./index_bg.wasm".into()],
            ..Default::default()
        },
        Box::new(NoopHook),
    );

    let mut entries = HashMap::new();
    entries.insert("shim.mjs".into(), FileName::Custom(shim_src));

    let bundles = bundler.bundle(entries)?;
    let writer = JsWriter::new(cm.clone(), "\n", &mut dst, None);
    let mut emitter = Emitter {
        cfg: Default::default(),
        wr: Box::new(writer),
        comments: None,
        cm,
    };

    emitter.emit_module(&bundles[0].module).map_err(Into::into)
}

struct BundleLoader(Arc<SourceMap>);

impl Load for BundleLoader {
    fn load(&self, file: &FileName) -> Result<ModuleData, anyhow::Error> {
        let (filename, src) = match file {
            FileName::Real(path) => (path.to_owned().into(), std::fs::read_to_string(path)?),
            FileName::Custom(src) => (file.clone(), src.clone()),
            _ => unreachable!(),
        };
        let fm = self.0.new_source_file(filename, src);
        let lexer = Lexer::new(
            Syntax::default(),
            EsVersion::Es2020,
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .expect("Failed to parse generated code");

        Ok(ModuleData {
            fm,
            module,
            helpers: Default::default(),
        })
    }
}

struct BundleResolver(PathBuf);

impl Resolve for BundleResolver {
    fn resolve(&self, _: &FileName, module_specifier: &str) -> Result<FileName, anyhow::Error> {
        if module_specifier.starts_with('.') {
            let path = self.0.join(module_specifier).canonicalize()?;
            return Ok(FileName::Real(path));
        }

        // In the future we could /potentially/ allow for users to resolve things in node_modules
        // if they want to use some package.
        // https://docs.rs/swc_ecma_loader/latest/swc_ecma_loader/resolvers/node/struct.NodeModulesResolver.html
        anyhow::bail!("workers-rs currently does not support non-relative imports");
    }
}

struct NoopHook;

impl Hook for NoopHook {
    fn get_import_meta_props(
        &self,
        _span: swc::common::Span,
        _module_record: &swc_bundler::ModuleRecord,
    ) -> Result<Vec<KeyValueProp>, anyhow::Error> {
        unreachable!()
    }
}
