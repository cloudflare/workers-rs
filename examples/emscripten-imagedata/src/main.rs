//! Edge image sanitisation for multimodal model input, built for
//! `wasm32-unknown-emscripten` with wasm SIMD.
//!
//! Natively this is a CLI over the same pipeline for reference timings.

mod pipeline;

#[cfg(target_arch = "wasm32")]
mod worker_entry;

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use std::time::Instant;

    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .expect("usage: emscripten-imagedata <image> [iterations]");
    let iters: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(10);
    let bytes = std::fs::read(&path).expect("read image");
    let b64 = pipeline::encode_base64(&bytes);

    for stage in [
        pipeline::Stage::Sniff,
        pipeline::Stage::Decode,
        pipeline::Stage::Resize,
        pipeline::Stage::Encode,
    ] {
        let opts = pipeline::Options {
            stage,
            ..Default::default()
        };
        let mut best = f64::MAX;
        let mut info = None;
        for _ in 0..iters {
            let start = Instant::now();
            let input = pipeline::decode_base64(&b64).expect("base64");
            let out = pipeline::process(&input, &opts).expect("process");
            let _ = pipeline::encode_base64(&out.jpeg);
            best = best.min(start.elapsed().as_secs_f64() * 1e3);
            info = Some(out.info);
        }
        println!(
            "{stage:?}: {best:.2} ms  {}",
            serde_json::to_string(&info.unwrap()).unwrap()
        );
    }
}
