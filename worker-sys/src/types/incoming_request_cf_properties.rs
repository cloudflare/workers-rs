use wasm_bindgen::prelude::*;

use crate::types::TlsClientAuth;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=IncomingRequestCfProperties)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type IncomingRequestCfProperties;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=colo)]
    pub fn colo(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=asn)]
    pub fn asn(this: &IncomingRequestCfProperties) -> u32;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=country)]
    pub fn country(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=httpProtocol)]
    pub fn http_protocol(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=requestPriority)]
    pub fn request_priority(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=tlsClientAuth)]
    pub fn tls_client_auth(this: &IncomingRequestCfProperties) -> Option<TlsClientAuth>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=tlsCipher)]
    pub fn tls_cipher(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=tlsVersion)]
    pub fn tls_version(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=city)]
    pub fn city(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=continent)]
    pub fn continent(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=latitude)]
    pub fn latitude(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=longitude)]
    pub fn longitude(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=postalCode)]
    pub fn postal_code(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=metroCode)]
    pub fn metro_code(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=region)]
    pub fn region(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=regionCode)]
    pub fn region_code(this: &IncomingRequestCfProperties) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=timezone)]
    pub fn timezone(this: &IncomingRequestCfProperties) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=isEUCountry)]
    pub fn is_eu_country(this: &IncomingRequestCfProperties) -> Option<String>;
}
