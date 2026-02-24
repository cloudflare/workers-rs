use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type BotManagement;

    #[wasm_bindgen(method, getter)]
    pub fn score(this: &BotManagement) -> u32;

    #[wasm_bindgen(method, getter, js_name=verifiedBot)]
    pub fn verified_bot(this: &BotManagement) -> bool;

    #[wasm_bindgen(method, getter, js_name=corporateProxy)]
    pub fn corporate_proxy(this: &BotManagement) -> bool;

    #[wasm_bindgen(method, getter, js_name=staticResource)]
    pub fn static_resource(this: &BotManagement) -> bool;

    #[wasm_bindgen(method, getter, js_name=ja3Hash)]
    pub fn ja3_hash(this: &BotManagement) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    pub fn ja4(this: &BotManagement) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name=jsDetection)]
    pub fn js_detection(this: &BotManagement) -> Option<JsDetection>;

    #[wasm_bindgen(method, getter, js_name=detectionIds)]
    pub fn detection_ids(this: &BotManagement) -> Vec<u32>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type JsDetection;

    #[wasm_bindgen(method, getter)]
    pub fn passed(this: &JsDetection) -> bool;
}
