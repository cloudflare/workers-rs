use wasm_bindgen::prelude::*;

use crate::types::TlsClientAuth;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type IncomingRequestCfProperties;

    #[wasm_bindgen(method, catch, getter)]
    pub fn colo(this: &IncomingRequestCfProperties) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn asn(this: &IncomingRequestCfProperties) -> Result<u32, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=asOrganization)]
    pub fn as_organization(this: &IncomingRequestCfProperties) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn country(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=httpProtocol)]
    pub fn http_protocol(this: &IncomingRequestCfProperties) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=requestPriority)]
    pub fn request_priority(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=tlsClientAuth)]
    pub fn tls_client_auth(
        this: &IncomingRequestCfProperties,
    ) -> Result<Option<TlsClientAuth>, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=tlsCipher)]
    pub fn tls_cipher(this: &IncomingRequestCfProperties) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=tlsVersion)]
    pub fn tls_version(this: &IncomingRequestCfProperties) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn city(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn continent(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn latitude(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn longitude(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=postalCode)]
    pub fn postal_code(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=metroCode)]
    pub fn metro_code(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn region(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=regionCode)]
    pub fn region_code(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;

    #[wasm_bindgen(method, catch, getter)]
    pub fn timezone(this: &IncomingRequestCfProperties) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=isEUCountry)]
    pub fn is_eu_country(this: &IncomingRequestCfProperties) -> Result<Option<String>, JsValue>;
}
