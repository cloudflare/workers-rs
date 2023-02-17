use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name=AbortSignal)]
        pub type AbortSignalExt;

        #[wasm_bindgen(structural, method, getter, js_class=AbortSignal, js_name=reason)]
        pub fn reason(this: &AbortSignalExt) -> JsValue;

        #[wasm_bindgen(static_method_of=AbortSignalExt, js_name=abort)]
        pub fn abort(reason: JsValue) -> web_sys::AbortSignal;
    }
}

pub trait AbortSignalExt {
    fn reason(&self) -> JsValue;

    fn abort(reason: JsValue) -> web_sys::AbortSignal;
}

impl AbortSignalExt for web_sys::AbortSignal {
    fn reason(&self) -> JsValue {
        self.unchecked_ref::<glue::AbortSignalExt>().reason()
    }

    fn abort(reason: JsValue) -> web_sys::AbortSignal {
        glue::AbortSignalExt::abort(reason)
    }
}
