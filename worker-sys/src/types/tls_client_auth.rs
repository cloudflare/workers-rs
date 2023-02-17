use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object, js_name=tlsClientAuth)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type TlsClientAuth;

    #[wasm_bindgen(structural, method, getter, js_name=certIssuerDNLegacy, js_class = "tlsClientAuth")]
    pub fn cert_issuer_dn_legacy(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certIssuerDN, js_class = "tlsClientAuth")]
    pub fn cert_issuer_dn(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certIssuerDNRFC2253, js_class = "tlsClientAuth")]
    pub fn cert_issuer_dn_rfc2253(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certSubjectDNLegacy, js_class = "tlsClientAuth")]
    pub fn cert_subject_dn_legacy(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certVerified, js_class = "tlsClientAuth")]
    pub fn cert_verified(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certNotAfter, js_class = "tlsClientAuth")]
    pub fn cert_not_after(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certSubjectDN, js_class = "tlsClientAuth")]
    pub fn cert_subject_dn(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certFingerprintSHA1, js_class = "tlsClientAuth")]
    pub fn cert_fingerprint_sha1(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certNotBefore, js_class = "tlsClientAuth")]
    pub fn cert_not_before(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certSerial, js_class = "tlsClientAuth")]
    pub fn cert_serial(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certPresented, js_class = "tlsClientAuth")]
    pub fn cert_presented(this: &TlsClientAuth) -> String;

    #[wasm_bindgen(structural, method, getter, js_name=certSubjectDNRFC2253, js_class = "tlsClientAuth")]
    pub fn cert_subject_dn_rfc2253(this: &TlsClientAuth) -> String;
}
