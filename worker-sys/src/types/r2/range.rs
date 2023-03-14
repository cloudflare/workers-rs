use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct R2Range {
    pub offset: Option<u32>,
    pub length: Option<u32>,
    pub suffix: Option<u32>,
}
