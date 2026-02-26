use crate::{
    error::Error,
    send::{SendFuture, SendWrapper},
    EnvBinding, Result,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// A binding to a Cloudflare Secret Store secret.
///
/// Each `[[secrets_store_secrets]]` entry in your wrangler.toml maps a single
/// secret (identified by `store_id` + `secret_name`) to a binding name.
/// Use [`SecretStore::get`] to retrieve the secret value.
#[derive(Debug, Clone)]
pub struct SecretStore(SendWrapper<worker_sys::SecretStoreSys>);

// Workers will never allow multithreading.
unsafe impl Send for SecretStore {}
unsafe impl Sync for SecretStore {}

impl EnvBinding for SecretStore {
    const TYPE_NAME: &'static str = "Fetcher";
}

impl JsCast for SecretStore {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<worker_sys::SecretStoreSys>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(SendWrapper::new(val.unchecked_into()))
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl AsRef<JsValue> for SecretStore {
    fn as_ref(&self) -> &JsValue {
        self.0.as_ref()
    }
}

impl From<JsValue> for SecretStore {
    fn from(val: JsValue) -> Self {
        Self::unchecked_from_js(val)
    }
}

impl From<SecretStore> for JsValue {
    fn from(secret_store: SecretStore) -> Self {
        let sys_obj: &worker_sys::SecretStoreSys = secret_store.0.as_ref();
        sys_obj.clone().into()
    }
}

impl SecretStore {
    /// Get a secret value from the secret store.
    ///
    /// Returns `Ok(None)` if the secret doesn't exist,
    /// or propagates any JS error that occurs.
    pub async fn get(&self) -> Result<Option<String>> {
        let promise = self.0.get().map_err(Error::from)?;

        let fut = SendFuture::new(JsFuture::from(promise));

        let output = fut.await.map_err(Error::from)?;

        if output.is_null() || output.is_undefined() {
            Ok(None)
        } else {
            Ok(Some(::serde_wasm_bindgen::from_value(output)?))
        }
    }
}
