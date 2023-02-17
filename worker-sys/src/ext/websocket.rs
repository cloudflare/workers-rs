use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name=WebSocket)]
        pub type WebSocketExt;

        #[wasm_bindgen(catch, structural, method, js_class=WebSocket, js_name=accept)]
        pub fn accept(this: &WebSocketExt) -> Result<(), JsValue>;
    }
}

pub trait WebSocketExt {
    /// Accepts the server side of the WebSocket.
    ///
    /// [CF Documentation](https://developers.cloudflare.com/workers/runtime-apis/websockets#accept)
    fn accept(&self) -> Result<(), JsValue>;
}

impl WebSocketExt for web_sys::WebSocket {
    fn accept(&self) -> Result<(), JsValue> {
        self.unchecked_ref::<glue::WebSocketExt>().accept()
    }
}
