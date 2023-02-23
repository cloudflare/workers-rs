use wasm_bindgen::prelude::*;

mod glue {
    use super::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type CacheStorage;

        #[wasm_bindgen(method, getter)]
        pub fn default(this: &CacheStorage) -> web_sys::Cache;
    }
}

pub trait CacheStorageExt {
    fn default(&self) -> web_sys::Cache;
}

impl CacheStorageExt for web_sys::CacheStorage {
    fn default(&self) -> web_sys::Cache {
        self.unchecked_ref::<glue::CacheStorage>().default()
    }
}
