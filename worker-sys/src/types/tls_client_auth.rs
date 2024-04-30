use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type TlsClientAuth;

    #[wasm_bindgen(method, catch, getter, js_name=certIssuerDNLegacy)]
    pub fn cert_issuer_dn_legacy(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certIssuerDN)]
    pub fn cert_issuer_dn(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certIssuerDNRFC2253)]
    pub fn cert_issuer_dn_rfc2253(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certSubjectDNLegacy)]
    pub fn cert_subject_dn_legacy(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certVerified)]
    pub fn cert_verified(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certNotAfter)]
    pub fn cert_not_after(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certSubjectDN)]
    pub fn cert_subject_dn(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certFingerprintSHA1)]
    pub fn cert_fingerprint_sha1(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certNotBefore)]
    pub fn cert_not_before(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certSerial)]
    pub fn cert_serial(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certPresented)]
    pub fn cert_presented(this: &TlsClientAuth) -> Result<String, JsValue>;

    #[wasm_bindgen(method, catch, getter, js_name=certSubjectDNRFC2253)]
    pub fn cert_subject_dn_rfc2253(this: &TlsClientAuth) -> Result<String, JsValue>;
}
