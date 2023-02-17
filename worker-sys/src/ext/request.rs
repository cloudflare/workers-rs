use wasm_bindgen::prelude::*;

use crate::types::IncomingRequestCfProperties;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name=Request)]
        pub type RequestExt;

        #[wasm_bindgen(structural, method, getter, js_class=Request, js_name=cf)]
        pub fn cf(this: &RequestExt) -> IncomingRequestCfProperties;
    }
}

pub trait RequestExt {
    /// Get the Cloudflare Properties from this request
    fn cf(&self) -> IncomingRequestCfProperties;
}

impl RequestExt for web_sys::Request {
    fn cf(&self) -> IncomingRequestCfProperties {
        self.unchecked_ref::<glue::RequestExt>().cf()
    }
}
