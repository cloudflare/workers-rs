use std::process::{Command, Stdio};

use anyhow::Result;
use libtest_mimic::{Arguments, Trial};
use wasmparser::{Parser, Payload};

fn main() -> Result<()> {
    // TODO: invoke worker-build with `--cfg worker_test`

    let args = Arguments::from_args();
    let exports = find_test_exports()?;
    let trials = exports
        .into_iter()
        .map(|export| {
            Trial::test(export.clone(), move || {
                run_test(&export, args.nocapture)?;

                Ok(())
            })
        })
        .collect();

    libtest_mimic::run(&args, trials).exit();
}

fn find_test_exports() -> Result<Vec<String>> {
    let bytes = std::fs::read("build/worker/index.wasm")?;
    let exports = Parser::new(0)
        .parse_all(&bytes)
        .filter_map(Result::ok)
        .find_map(|payload| match payload {
            Payload::ExportSection(exports) => Some(exports),
            _ => None,
        })
        .expect("no export section found");

    let mut test_exports = Vec::new();

    for export in exports {
        let export = export?;
        const PREFIX: &str = "__worker_test_";
        if export.name.starts_with(PREFIX) {
            test_exports.push(export.name[PREFIX.len()..].to_string());
        }
    }

    Ok(test_exports)
}

fn run_test(name: &str, nocapture: bool) -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("runner.cjs");

    std::fs::write(&path, include_bytes!("runner.cjs"))?;

    let mut child = Command::new("node")
        .args(["--experimental-vm-modules", path.to_str().unwrap(), name])
        .env("NODE_NO_WARNINGS", "1")
        .stdout(if nocapture {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .stderr(if nocapture {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .spawn()?;

    let exit_status = child.wait()?;

    if !exit_status.success() {
        anyhow::bail!("test failed");
    }

    Ok(())
}
