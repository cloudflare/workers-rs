use js_sys::Math;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(
    inline_js = "export function someUniqueValue() { return \"f408ee6cdad87526c64656984fb6637d09203ca4\"; }"
)]
extern "C" {
    // A function that returns a very unique random string of characters that we can use to identify
    // if our snippets are getting included in the final bundle.
    #[wasm_bindgen(js_name = "someUniqueValue")]
    fn some_unique_value() -> String;
}

#[wasm_bindgen(js_name = "addRandom")]
pub fn add_random(x: f64) -> f64 {
    // We need to call our function so it doesn't get optimized out by rustc/wasm-bindgen/SWC.
    let _ = some_unique_value();

    // Called to get wasm-bindgen to generate an exported function that we can check.
    js_sys::global();

    // Math.random is used because wasm-bindgen generates a exported constant that is set to
    // Math.random
    x + Math::random()
}
