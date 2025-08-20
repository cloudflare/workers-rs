use wasm_bindgen::prelude::*;

use crate::types::IncomingRequestCfProperties;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type Request;

        #[wasm_bindgen(method, catch, getter)]
        pub fn cf(this: &Request) -> Result<Option<IncomingRequestCfProperties>, JsValue>;
    }
}

pub trait RequestExt {
    /// Get the Cloudflare Properties from this request
    fn cf(&self) -> Option<IncomingRequestCfProperties>;
}

impl RequestExt for web_sys::Request {
    fn cf(&self) -> Option<IncomingRequestCfProperties> {
        self.unchecked_ref::<glue::Request>().cf().unwrap()
    }
}
