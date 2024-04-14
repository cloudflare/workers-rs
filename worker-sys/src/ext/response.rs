use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type Response;

        #[wasm_bindgen(method, catch, getter)]
        pub fn webSocket(this: &Response) -> Result<Option<web_sys::WebSocket>, JsValue>;
    }
}

pub trait ResponseExt {
    /// Getter for the `webSocket` field of this object.
    fn websocket(&self) -> Option<web_sys::WebSocket>;
}

impl ResponseExt for web_sys::Response {
    fn websocket(&self) -> Option<web_sys::WebSocket> {
        self.unchecked_ref::<glue::Response>().webSocket().unwrap()
    }
}
