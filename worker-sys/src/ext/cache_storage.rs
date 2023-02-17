use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_name=CacheStorage)]
        pub type CacheStorageExt;

        #[wasm_bindgen(method, structural, getter, js_class=CacheStorage)]
        pub fn default(this: &CacheStorageExt) -> web_sys::Cache;
    }
}

pub trait CacheStorageExt {
    fn default(&self) -> web_sys::Cache;
}

impl CacheStorageExt for web_sys::CacheStorage {
    fn default(&self) -> web_sys::Cache {
        self.unchecked_ref::<glue::CacheStorageExt>().default()
    }
}
