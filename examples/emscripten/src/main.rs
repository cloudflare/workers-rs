//! A Worker built for `wasm32-unknown-emscripten`: `std::fs` works against
//! Emscripten's in-memory filesystem, so code written around files runs
//! unchanged.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use worker::*;

fn main() {}

/// Word-frequency report over `input`, going through files the way a
/// command-line tool would.
fn report(dir: &Path, input: &str) -> std::io::Result<String> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("input.txt"), input)?;

    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    let mut lines = 0;
    for line in BufReader::new(fs::File::open(dir.join("input.txt"))?).lines() {
        lines += 1;
        for word in line?.split_whitespace() {
            *counts
                .entry(
                    word.trim_matches(|c: char| !c.is_alphanumeric())
                        .to_lowercase(),
                )
                .or_default() += 1;
        }
    }
    counts.remove("");

    let mut out = fs::File::create(dir.join("report.txt"))?;
    writeln!(out, "{lines} lines, {} distinct words", counts.len())?;
    let mut by_count: Vec<_> = counts.into_iter().collect();
    by_count.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    for (word, n) in by_count.iter().take(10) {
        writeln!(out, "{n:>4} {word}")?;
    }

    let mut listing = String::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        listing += &format!(
            "{} ({} bytes)\n",
            entry.file_name().to_string_lossy(),
            entry.metadata()?.len()
        );
    }
    Ok(format!(
        "{}\n{listing}",
        fs::read_to_string(dir.join("report.txt"))?
    ))
}

#[event(fetch)]
async fn fetch(mut req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let text = match req.method() {
        Method::Post => req.text().await?,
        _ => return Response::ok("POST some text to get a word-frequency report\n"),
    };
    let dir = std::env::temp_dir().join("wordcount");
    let out = report(&dir, &text).map_err(|e| Error::RustError(e.to_string()))?;
    fs::remove_dir_all(&dir).map_err(|e| Error::RustError(e.to_string()))?;
    Response::ok(out)
}
