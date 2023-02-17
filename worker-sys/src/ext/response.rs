use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name=Response)]
        pub type ResponseExt;

        #[wasm_bindgen(structural, method, getter, js_class=Response, js_name=webSocket)]
        pub fn websocket(this: &ResponseExt) -> Option<web_sys::WebSocket>;
    }
}

pub trait ResponseExt {
    /// Getter for the `webSocket` field of this object.
    fn websocket(&self) -> Option<web_sys::WebSocket>;
}

impl ResponseExt for web_sys::Response {
    fn websocket(&self) -> Option<web_sys::WebSocket> {
        self.unchecked_ref::<glue::ResponseExt>().websocket()
    }
}
