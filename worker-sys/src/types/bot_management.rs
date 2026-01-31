use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type BotManagement;

    #[wasm_bindgen(method, catch, getter, js_name=score)]
    pub fn score(this: &BotManagement) -> Result<usize, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=verifiedBot)]
    pub fn verified_bot(this: &BotManagement) -> Result<bool, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=staticResource)]
    pub fn static_resource(this: &BotManagement) -> Result<bool, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=ja3Hash)]
    pub fn ja3_hash(this: &BotManagement) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=ja4)]
    pub fn ja4(this: &BotManagement) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=jsDetection)]
    pub fn js_detection(this: &BotManagement) -> Result<JsDetection, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=detectionIds)]
    pub fn detection_ids(this: &BotManagement) -> Result<Vec<usize>, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type JsDetection;

    #[wasm_bindgen(method, catch, getter, js_name=passed)]
    pub fn passed(this: &JsDetection) -> Result<bool, JsValue>;
}
