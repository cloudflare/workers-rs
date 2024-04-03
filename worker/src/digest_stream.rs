use wasm_bindgen::{self, prelude::*};
use wasm_bindgen_futures;
use web_sys::js_sys::{ArrayBuffer, Uint8Array};
use web_sys::WritableStream;

impl DigestStream {
    #[allow(dead_code)]
    pub fn new(algo: DigestStreamAlgorithm) -> Self {
        Self::internal_new(algo.as_str())
    }

    #[allow(dead_code)]
    pub async fn digest(&self) -> Result<Uint8Array, JsValue> {
        let buffer: ArrayBuffer = self.internal_digest().await?.unchecked_into();
        Ok(Uint8Array::new(&buffer))
    }
}

// from https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/digest#syntax
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DigestStreamAlgorithm {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

#[wasm_bindgen]
extern "C" {
    /// A Rust-friendly wrapper around the non-standard [crypto.DigestStream](https://developers.cloudflare.com/workers/runtime-apis/web-crypto/#constructors) API
    ///
    /// Example usage:
    /// ```rust
    /// let digest_stream = DigestStream::new(DigestStreamAlgorithm::Sha256);
    ///
    /// let mut req_init = web_sys::RequestInit::new();
    /// req_init.method("POST");
    /// req_init.body(Some(&JsValue::from_str("foo")));
    /// let req = web_sys::Request::new_with_str_and_init("http://internal", &req_init).unwrap();
    ///
    /// let body = req.body().unwrap();
    /// // just kick the promise off to the background, we'll await the digest itself
    /// let _ = body.pipe_to(&digest_stream);
    ///
    /// let bytes:Vec<u8> = digest_stream.digest().await.unwrap().to_vec();
    /// ```
    #[wasm_bindgen(extends = WritableStream)]
    pub type DigestStream;

    #[wasm_bindgen(constructor, js_namespace = crypto, js_name = "new")]
    fn internal_new(algorithm: &str) -> DigestStream;

    #[wasm_bindgen(catch, method, getter, js_name = "digest")]
    async fn internal_digest(this: &DigestStream) -> Result<JsValue, JsValue>;
}

impl DigestStreamAlgorithm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sha1 => "SHA-1",
            Self::Sha256 => "SHA-256",
            Self::Sha384 => "SHA-384",
            Self::Sha512 => "SHA-512",
        }
    }
}
