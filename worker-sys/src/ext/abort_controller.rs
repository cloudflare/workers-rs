use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type AbortController;

        #[wasm_bindgen(method, catch, js_name=abort)]
        pub fn abort_with_reason(this: &AbortController, reason: &JsValue) -> Result<(), JsValue>;
    }
}

pub trait AbortControllerExt {
    fn abort_with_reason(&self, reason: &JsValue);
}

impl AbortControllerExt for web_sys::AbortController {
    fn abort_with_reason(&self, reason: &JsValue) {
        self.unchecked_ref::<glue::AbortController>()
            .abort_with_reason(reason)
            .unwrap()
    }
}
