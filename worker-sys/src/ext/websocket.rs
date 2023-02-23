use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type WebSocket;

        #[wasm_bindgen(method, catch)]
        pub fn accept(this: &WebSocket) -> Result<(), JsValue>;
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
        self.unchecked_ref::<glue::WebSocket>().accept()
    }
}
