use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=AnalyticsEngineDataset)]
    #[derive(Debug, Clone)]
    pub type AnalyticsEngineDataset;

    #[wasm_bindgen(method, catch, js_name=writeDataPoint)]
    pub fn write_data_point(this: &AnalyticsEngineDataset, event: JsValue) -> Result<(), JsValue>;
}
