use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::send::SendFuture;

/// A Rust-friendly wrapper around the non-standard [crypto.DigestStream](https://developers.cloudflare.com/workers/runtime-apis/web-crypto/#constructors) API
///
/// Example usage:
/// ```rust
/// let digest_stream = DigestStream::new(DigestStreamAlgorithm::Sha256);
///
/// // create a ReadableStream from a string
/// let mut req_init = web_sys::RequestInit::new();
/// req_init.method("POST");
/// req_init.body(Some(&JsValue::from_str("foo")));
/// let req = web_sys::Request::new_with_str_and_init("http://internal", &req_init).unwrap();
/// let body = req.body().unwrap();
///
/// // just kick the promise off to the background, we'll await the digest itself
/// // since this is piped to a JS readable stream, call .raw() to get the underlying JS object
/// let _ = body.pipe_to(digest_stream.raw());
///
/// let bytes:Vec<u8> = digest_stream.digest().await.unwrap().to_vec();
/// ```
pub struct DigestStream {
    inner: worker_sys::DigestStream,
}

impl DigestStream {
    pub fn new(algo: DigestStreamAlgorithm) -> Self {
        Self {
            inner: worker_sys::DigestStream::new(algo.as_str()),
        }
    }

    pub async fn digest(&self) -> Result<Uint8Array, crate::Error> {
        let fut = SendFuture::new(JsFuture::from(self.inner.digest()));
        let buffer: ArrayBuffer = fut.await?.unchecked_into();
        Ok(Uint8Array::new(&buffer))
    }

    pub fn raw(&self) -> &worker_sys::DigestStream {
        &self.inner
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
