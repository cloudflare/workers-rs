use crate::{
    send::{SendFuture, SendWrapper},
    EnvBinding, Fetcher, Result,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

mod sys {
    #[wasm_bindgen::prelude::wasm_bindgen]
    extern "C" {
        #[derive(Clone)]
        #[wasm_bindgen::prelude::wasm_bindgen(extends = js_sys::Object)]
        pub type SecretStoreSys;
        #[wasm_bindgen::prelude::wasm_bindgen(method, catch, js_name = "get")]
        pub fn get(
            this: &SecretStoreSys,
        ) -> std::result::Result<js_sys::Promise, wasm_bindgen::JsValue>;
    }
}

/// A binding to a Cloudflare Secret Store.
#[derive(Clone)]
pub struct SecretStore(SendWrapper<sys::SecretStoreSys>);

// Allows for attachment to axum router, as Workers will never allow multithreading.
unsafe impl Send for SecretStore {}
unsafe impl Sync for SecretStore {}

impl EnvBinding for SecretStore {
    const TYPE_NAME: &'static str = "Fetcher";
}

impl JsCast for SecretStore {
    fn instanceof(val: &wasm_bindgen::JsValue) -> bool {
        Fetcher::instanceof(val)
    }

    fn unchecked_from_js(val: wasm_bindgen::JsValue) -> Self {
        let fetcher = Fetcher::unchecked_from_js(val);
        Self::from(fetcher)
    }

    fn unchecked_from_js_ref(val: &wasm_bindgen::JsValue) -> &Self {
        unsafe { &*(val as *const wasm_bindgen::JsValue as *const Self) }
    }
}

impl AsRef<wasm_bindgen::JsValue> for SecretStore {
    fn as_ref(&self) -> &wasm_bindgen::JsValue {
        self.0.as_ref()
    }
}

impl From<wasm_bindgen::JsValue> for SecretStore {
    fn from(val: wasm_bindgen::JsValue) -> Self {
        Self::unchecked_from_js(val)
    }
}

impl From<Fetcher> for SecretStore {
    fn from(fetcher: Fetcher) -> Self {
        Self(SendWrapper::new(fetcher.into_rpc()))
    }
}

impl From<SecretStore> for wasm_bindgen::JsValue {
    fn from(secret_store: SecretStore) -> Self {
        let sys_obj: &sys::SecretStoreSys = secret_store.0.as_ref();
        sys_obj.clone().into()
    }
}

impl SecretStore {
    /// Get a secret value from the secret store.
    /// Returns None if the secret doesn't exist.
    pub async fn get(&self) -> Result<Option<String>> {
        let promise = match self.0.get() {
            Ok(p) => p,
            Err(_) => return Ok(None), // Secret not found
        };

        let fut = SendFuture::new(JsFuture::from(promise));

        let output = match fut.await {
            Ok(val) => val,
            Err(_) => return Ok(None), // Promise rejected, secret not found
        };

        if output.is_null() || output.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(::serde_wasm_bindgen::from_value(output)?))
        }
    }
}
