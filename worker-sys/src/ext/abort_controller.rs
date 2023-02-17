use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name=AbortController)]
        pub type AbortControllerExt;

        #[wasm_bindgen(method, structural, js_class=AbortController, js_name=abort)]
        pub fn abort_with_reason(this: &AbortControllerExt, reason: JsValue);
    }
}

pub trait AbortControllerExt {
    fn abort_with_reason(&self, reason: JsValue);
}

impl AbortControllerExt for web_sys::AbortController {
    fn abort_with_reason(&self, reason: JsValue) {
        self.unchecked_ref::<glue::AbortControllerExt>().abort_with_reason(reason)
    }
}
