fn main() {
    // Set by `worker-build --emscripten --tokio`.
    println!("cargo:rustc-check-cfg=cfg(worker_tokio, values(\"event_loop\"))");
}
