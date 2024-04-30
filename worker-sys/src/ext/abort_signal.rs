use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type AbortSignal;

        #[wasm_bindgen(method, catch, getter)]
        pub fn reason(this: &AbortSignal) -> Result<JsValue, JsValue>;

        #[wasm_bindgen(static_method_of=AbortSignal, catch)]
        pub fn abort() -> Result<web_sys::AbortSignal, JsValue>;

        #[wasm_bindgen(static_method_of=AbortSignal, js_name=abort, catch)]
        pub fn abort_with_reason(reason: &JsValue) -> Result<web_sys::AbortSignal, JsValue>;
    }
}

pub trait AbortSignalExt {
    fn reason(&self) -> JsValue;

    fn abort() -> web_sys::AbortSignal;

    fn abort_with_reason(reason: &JsValue) -> web_sys::AbortSignal;
}

impl AbortSignalExt for web_sys::AbortSignal {
    fn reason(&self) -> JsValue {
        self.unchecked_ref::<glue::AbortSignal>().reason().unwrap()
    }

    fn abort() -> web_sys::AbortSignal {
        glue::AbortSignal::abort().unwrap()
    }

    fn abort_with_reason(reason: &JsValue) -> web_sys::AbortSignal {
        glue::AbortSignal::abort_with_reason(reason).unwrap()
    }
}
