use crate::utils;
use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn off_in_dev() {
    let fixture = utils::fixture::Fixture::new();
    fixture.readme().cargo_toml("foo").file("src/lib.rs", "");
    fixture.install_local_wasm_bindgen();
    fixture.install_wasm_opt();

    fixture
        .wasm_pack()
        .arg("build")
        .arg("--dev")
        .assert()
        .stderr(predicates::str::contains("wasm-opt").not())
        .success();
}

#[test]
fn on_in_release() {
    let fixture = utils::fixture::Fixture::new();
    fixture.readme().cargo_toml("foo").file("src/lib.rs", "");
    fixture.install_local_wasm_bindgen();
    fixture.install_wasm_opt();

    fixture
        .wasm_pack()
        .arg("build")
        .assert()
        .stderr(predicates::str::contains("wasm-opt"))
        .success();
}

#[test]
fn disable_in_release() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                authors = []
                description = ""
                license = "MIT"
                name = "foo"
                repository = ""
                version = "0.1.0"

                [lib]
                crate-type = ["cdylib"]

                [dependencies]
                wasm-bindgen = "0.2"

                [package.metadata.wasm-pack.profile.release]
                wasm-opt = false
            "#,
        )
        .file("src/lib.rs", "");
    fixture.install_local_wasm_bindgen();
    fixture.install_wasm_opt();

    fixture
        .wasm_pack()
        .arg("build")
        .assert()
        .stderr(predicates::str::contains("wasm-opt").not())
        .success();
}

#[test]
fn enable_in_dev() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                authors = []
                description = ""
                license = "MIT"
                name = "foo"
                repository = ""
                version = "0.1.0"

                [lib]
                crate-type = ["cdylib"]

                [dependencies]
                wasm-bindgen = "0.2"

                [package.metadata.wasm-pack.profile.dev]
                wasm-opt = true
            "#,
        )
        .file("src/lib.rs", "");
    fixture.install_local_wasm_bindgen();
    fixture.install_wasm_opt();

    fixture
        .wasm_pack()
        .arg("build")
        .arg("--dev")
        .assert()
        .stderr(predicates::str::contains(
            "Optimizing wasm binaries with `wasm-opt`",
        ))
        .success();
}

#[test]
fn custom_args() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                authors = []
                description = ""
                license = "MIT"
                name = "foo"
                repository = ""
                version = "0.1.0"

                [lib]
                crate-type = ["cdylib"]

                [dependencies]
                wasm-bindgen = "0.2"

                [package.metadata.wasm-pack.profile.release]
                wasm-opt = ['--not-accepted-argument']
            "#,
        )
        .file("src/lib.rs", "");
    fixture.install_local_wasm_bindgen();
    fixture.install_wasm_opt();

    fixture
        .wasm_pack()
        .arg("build")
        .assert()
        .stderr(predicates::str::contains("--not-accepted-argument"))
        .failure();
}

#[test]
fn misconfigured() {
    let fixture = utils::fixture::Fixture::new();
    fixture
        .readme()
        .file(
            "Cargo.toml",
            r#"
                [package]
                authors = []
                description = ""
                license = "MIT"
                name = "foo"
                repository = ""
                version = "0.1.0"

                [lib]
                crate-type = ["cdylib"]

                [dependencies]
                wasm-bindgen = "0.2"

                [package.metadata.wasm-pack.profile.release]
                wasm-opt = 32
            "#,
        )
        .file("src/lib.rs", "");
    fixture.install_local_wasm_bindgen();
    fixture.install_wasm_opt();

    fixture
        .wasm_pack()
        .arg("build")
        .assert()
        .stderr(predicates::str::contains("failed to parse manifest"))
        .failure();
}
