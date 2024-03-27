use wasm_bindgen::prelude::*;

use crate::types::TlsClientAuth;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type IncomingRequestCfProperties;

    #[wasm_bindgen(method, getter)]
    pub fn colo(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn asn(this: &IncomingRequestCfProperties) -> u32;

    #[wasm_bindgen(method, getter, js_name=asOrganization)]
    pub fn as_organization(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn country(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name=httpProtocol)]
    pub fn http_protocol(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(method, getter, js_name=requestPriority)]
    pub fn request_priority(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name=tlsClientAuth)]
    pub fn tls_client_auth(this: &IncomingRequestCfProperties) -> Option<TlsClientAuth>;

    #[wasm_bindgen(method, getter, js_name=tlsCipher)]
    pub fn tls_cipher(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(method, getter, js_name=tlsVersion)]
    pub fn tls_version(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(method, getter)]
    pub fn city(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    pub fn continent(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    pub fn latitude(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    pub fn longitude(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name=postalCode)]
    pub fn postal_code(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name=metroCode)]
    pub fn metro_code(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    pub fn region(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name=regionCode)]
    pub fn region_code(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter)]
    pub fn timezone(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(method, getter, js_name=isEUCountry)]
    pub fn is_eu_country(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(method, getter, js_name=hostMetadata)]
    pub fn host_metadata(this: &IncomingRequestCfProperties) -> wasm_bindgen::JsValue;
}
