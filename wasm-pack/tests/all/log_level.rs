use crate::utils;
use assert_cmd::prelude::*;
use predicates::boolean::PredicateBooleanExt;
use predicates::prelude::predicate::str::contains;
use predicates::reflection::PredicateReflection;
use predicates::Predicate;
use wasm_pack::emoji;

fn matches_info() -> impl Predicate<str> + PredicateReflection {
    contains(format!("[INFO]: {}Checking for the Wasm target...", emoji::TARGET))
        .and(contains(format!("[INFO]: {}Compiling to Wasm...", emoji::CYCLONE)))
        .and(contains("[INFO]: License key is set in Cargo.toml but no LICENSE file(s) were found; Please add the LICENSE file(s) to your project directory"))
        .and(contains("[INFO]: Optimizing wasm binaries with `wasm-opt`..."))
        .and(contains(format!("[INFO]: {} Done in ", emoji::SPARKLE)))
        .and(contains(format!("[INFO]: {} Your wasm pkg is ready to publish at ", emoji::PACKAGE)))
}

fn matches_cargo() -> impl Predicate<str> + PredicateReflection {
    contains("Finished release [optimized] target(s) in ").or(contains(
        "Finished `release` profile [optimized] target(s) in ",
    ))
}

#[test]
fn quiet() {
    utils::fixture::Fixture::new()
        .cargo_toml("js-hello-world")
        .hello_world_src_lib()
        .wasm_pack()
        .arg("--quiet")
        .arg("build")
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn log_level_info() {
    utils::fixture::Fixture::new()
        .cargo_toml("js-hello-world")
        .hello_world_src_lib()
        .wasm_pack()
        .arg("--log-level")
        .arg("info")
        .arg("build")
        .assert()
        .success()
        .stdout("")
        .stderr(matches_cargo().and(matches_info()));
}

#[test]
fn log_level_warn() {
    utils::fixture::Fixture::new()
        .cargo_toml("js-hello-world")
        .hello_world_src_lib()
        .wasm_pack()
        .arg("--log-level")
        .arg("warn")
        .arg("build")
        .assert()
        .success()
        .stdout("")
        .stderr(matches_cargo().and(matches_info().not()));
}

#[test]
fn log_level_error() {
    utils::fixture::Fixture::new()
        .cargo_toml("js-hello-world")
        .hello_world_src_lib()
        .wasm_pack()
        .arg("--log-level")
        .arg("error")
        .arg("build")
        .assert()
        .success()
        .stdout("")
        .stderr(matches_cargo().and(matches_info().not()));
}
