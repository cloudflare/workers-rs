use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=IncomingRequestCfProperties)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Cf;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=colo)]
    pub fn colo(this: &Cf) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=asn)]
    pub fn asn(this: &Cf) -> u32;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=country)]
    pub fn country(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=httpProtocol)]
    pub fn http_protocol(this: &Cf) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=requestPriority)]
    pub fn request_priority(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=tlsClientAuth)]
    pub fn tls_client_auth(this: &Cf) -> Option<TlsClientAuth>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=tlsCipher)]
    pub fn tls_cipher(this: &Cf) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=tlsVersion)]
    pub fn tls_version(this: &Cf) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=city)]
    pub fn city(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=continent)]
    pub fn continent(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=latitude)]
    pub fn latitude(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=longitude)]
    pub fn longitude(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=postalCode)]
    pub fn postal_code(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=metroCode)]
    pub fn metro_code(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=region)]
    pub fn region(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=regionCode)]
    pub fn region_code(this: &Cf) -> Option<String>;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=timezone)]
    pub fn timezone(this: &Cf) -> String;

    #[wasm_bindgen(structural, method, getter, js_class=IncomingRequestCfProperties, js_name=isEUCountry)]
    pub fn is_eu_country(this: &Cf) -> Option<String>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=::js_sys::Object, js_name=tlsClientAuth)]
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

    #[wasm_bindgen(structural, method, getter, js_name=certSubjectDNRFC225, js_class = "tlsClientAuth")]
    pub fn cert_subject_dn_rfc225(this: &TlsClientAuth) -> String;
}
