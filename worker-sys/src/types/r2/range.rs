use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct R2Range {
    pub offset: Option<f64>,
    pub length: Option<f64>,
    pub suffix: Option<f64>,
}
