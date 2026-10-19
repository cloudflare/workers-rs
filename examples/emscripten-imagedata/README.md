# Emscripten image sanitisation example

> **Experimental Preview**

A Worker that normalises untrusted images into model input, built with
`worker-build --emscripten` for `wasm32-unknown-emscripten` with wasm SIMD.
It exists to measure whether edge-side image processing and sanitisation of
multimodal base64 payloads is viable inside a Worker.

```sh
git clone --recursive https://github.com/cloudflare/workers-rs
cd workers-rs/examples/emscripten-imagedata
npx wrangler dev
```

## What it does

For every image in a request it sniffs the real format (the declared media
type is ignored), rejects it from the header alone if it exceeds the pixel
budget, decodes it, applies the EXIF orientation, flattens alpha onto white,
downscales it to fit the longest edge, and re-encodes it as a baseline JPEG
with no metadata. Animated inputs yield their first frame.

```sh
# Multimodal message body: every data:image/... URL and every
# {"type":"base64","data":...} source is replaced in place.
curl -X POST -H 'content-type: application/json' --data @messages.json localhost:8787/

# Raw image bytes in, image/jpeg out.
curl -X POST -H 'content-type: image/png' --data-binary @photo.png localhost:8787/ -o out.jpg

# A data: URL or bare base64 string in, data: URL out.
curl -X POST --data "data:image/jpeg;base64,$(base64 -w0 photo.jpg)" localhost:8787/
```

Query parameters: `max` (longest edge, default 1568), `quality` (default 85),
and for benchmarking `stage=sniff|decode|resize|encode` to stop early and
`repeat=<n>` to run the pipeline n times per image.

Every response carries `x-image-info` (per-image dimensions and byte counts)
and `x-wasm-memory-bytes` (the wasm heap size, which never shrinks).

## SIMD

Accepted formats are JPEG, PNG, WebP and GIF. The pipeline is:

| Stage           | Crate                                                | wasm SIMD                              |
| --------------- | ---------------------------------------------------- | -------------------------------------- |
| base64          | `base64-simd`                                        | yes                                    |
| JPEG decode     | `zune-jpeg`                                          | autovectorisation only                 |
| WebP decode     | `libwebp-sys` ([patched](../../libwebp-sys))         | libwebp's SSE2 paths via Emscripten    |
| PNG, GIF decode | `image`                                              | autovectorisation only                 |
| resize          | `fast_image_resize`                                  | yes                                    |
| JPEG encode     | `jpeg-encoder` with `use_wide`                       | fDCT via `wide`                        |

The `simd128` target feature is enabled for the Rust crates through
`RUSTFLAGS` and for the C side and the emcc link through `EMCC_CFLAGS` (see
`wrangler.toml`). Wasm has no runtime feature detection, so every crate takes
its SIMD path from the compile-time target feature.

`libwebp-sys` is a git submodule at the repository root on a branch pending
an upstream pull request: an Emscripten build arm enabling libwebp's SSE2
code paths (lowered to simd128 by Emscripten) when the simd128 target
feature is set. No Rust WebP decoder has SIMD on any architecture, and
`image-webp` measured 181 ms for a 4 MP decode against libwebp's 110 ms.
`jpeg-encoder` is a git dependency on upstream for its unreleased `use_wide`
feature, which lowers its fDCT to simd128.

`zune-jpeg` is used unpatched: with `+simd128` LLVM autovectorises its
scalar IDCT and colour conversion. A hand-written simd128 port was measured
at a further 15% on decode and left out to stay on a stable toolchain.

## Benchmark

`bench/gen.py` writes photo-like fixtures (smooth gradients plus noise, so the
entropy coders do real work) and `bench/build.sh` builds `build-simd/` and
`build-scalar/`. `bench/run.sh simd|scalar` serves one with `wrangler dev`
and runs `bench/bench.mjs`, which reports per-stage milliseconds inside
workerd. Stages are cumulative, so the delta column is the cost of that stage.

`Date.now()` does not advance during synchronous execution in workerd, so the
harness amortises request overhead with `repeat` instead of timing in-Worker.

Numbers from one x86_64 machine (ms per image, `wrangler dev`, 12 MP JPEG
downscaled to 1568 px):

| stage           | native (AVX2) | wasm simd128 | wasm scalar |
| --------------- | ------------: | -----------: | ----------: |
| decode          |            64 |          117 |         157 |
| resize          |            11 |           31 |         139 |
| encode          |            11 |           18 |          21 |
| base64 (5.6 MB) |             3 |           15 |          18 |
| total           |            88 |          181 |         335 |

Run-to-run noise is around 10%. A 4 MP WebP decodes in 110 ms with libwebp's SSE2 paths, from 181 ms with the
pure-Rust `image-webp` decoder.

## Layout

Handlers live in `src/worker_entry.rs`; `src/pipeline.rs` is free of Worker
types and is what `cargo test` exercises natively. On native targets the
binary is a small CLI over the same pipeline for reference timings:

```sh
cargo run --release -- bench/fixtures/12mp.jpg
```

Emscripten's default 64 KiB stack is too small for the decoders, so the build
passes `-sSTACK_SIZE=1048576`; with `-sASSERTIONS=0` a stack overflow is a
silent `unreachable` trap rather than a diagnostic.

See the [emscripten](../emscripten) example for the toolchain layout.
