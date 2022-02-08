use js_sys::Math;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = "addRandom")]
pub fn add_random(x: f64) -> f64 {
    // Called to get wasm-bindgen to generate an exported function that we can check.
    js_sys::global();

    // Math.random is used because wasm-bindgen generates a exported constant that is set to
    // Math.random
    x + Math::random()
}
