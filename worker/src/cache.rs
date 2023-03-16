use serde::Serialize;
use wasm_bindgen::JsCast;
use worker_sys::ext::CacheStorageExt;

use crate::{
    body::Body,
    futures::SendJsFuture,
    http::{request, response},
    Result,
};

/// Provides access to the [Cache API](https://developers.cloudflare.com/workers/runtime-apis/cache).
/// Because `match` is a reserved keyword in Rust, the `match` method has been renamed to `get`.
///
/// Our implementation of the Cache API respects the following HTTP headers on the response passed to `put()`:
///
/// - `Cache-Control`
///   - Controls caching directives.
///     This is consistent with [Cloudflare Cache-Control Directives](https://developers.cloudflare.com/cache/about/cache-control#cache-control-directives).
///     Refer to [Edge TTL](https://developers.cloudflare.com/cache/how-to/configure-cache-status-code#edge-ttl) for a list of HTTP response codes and their TTL when Cache-Control directives are not present.
/// - `Cache-Tag`
///   - Allows resource purging by tag(s) later (Enterprise only).
/// - `ETag`
///   - Allows `cache.get()` to evaluate conditional requests with If-None-Match.
/// - `Expires`
///   - A string that specifies when the resource becomes invalid.
/// - `Last-Modified`
///   - Allows `cache.get()` to evaluate conditional requests with If-Modified-Since.
///
/// This differs from the web browser Cache API as they do not honor any headers on the request or response.
///
/// Responses with `Set-Cookie` headers are never cached, because this sometimes indicates that the response contains unique data. To store a response with a `Set-Cookie` header, either delete that header or set `Cache-Control: private=Set-Cookie` on the response before calling `cache.put()`.
///
/// Use the `Cache-Control` method to store the response without the `Set-Cookie` header.
#[derive(Debug)]
pub struct Cache {
    inner: web_sys::Cache,
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}

impl Default for Cache {
    fn default() -> Self {
        let global: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();

        Self {
            inner: global.caches().unwrap().default(),
        }
    }
}

impl Cache {
    /// Opens a [`Cache`] by name. To access the default global cache, use [`Cache::default()`](`Default::default`).
    pub async fn open(name: String) -> Self {
        let fut = {
            let global: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
            let cache = global.caches().unwrap().open(&name);

            SendJsFuture::from(cache)
        };

        // unwrap is safe because this promise never rejects
        // https://developer.mozilla.org/en-US/docs/Web/API/CacheStorage/open
        let inner = fut.await.unwrap().into();
        Self { inner }
    }

    /// Adds to the cache a [`Response`] keyed to the given request.
    ///
    /// The [`Response`] should include a `cache-control` header with `max-age` or `s-maxage` directives,
    /// otherwise the Cache API will not cache the response.
    /// The `stale-while-revalidate` and `stale-if-error` directives are not supported
    /// when using the `cache.put` or `cache.get` methods.
    /// For more information about how the Cache works, visit the documentation at [Cache API](https://developers.cloudflare.com/workers/runtime-apis/cache/)
    /// and [Cloudflare Cache-Control Directives](https://developers.cloudflare.com/cache/about/cache-control#cache-control-directives)
    ///
    /// The Cache API will throw an error if:
    /// - the request passed is a method other than GET.
    /// - the response passed has a status of 206 Partial Content.
    /// - the response passed contains the header `Vary: *` (required by the Cache API specification).
    pub async fn put<K: Into<CacheKey>>(&self, key: K, res: impl Into<CacheValue>) -> Result<()> {
        let fut = {
            let promise = match key.into() {
                CacheKey::Url(url) => self.inner.put_with_str(url.as_str(), &res.into().0),
                CacheKey::Request(req) => self.inner.put_with_request(&req, &res.into().0),
            };

            SendJsFuture::from(promise)
        };

        fut.await?;
        Ok(())
    }

    /// Returns the [`Response`] object keyed to that request. Never sends a subrequest to the origin. If no matching response is found in cache, returns `None`.
    ///
    /// Unlike the browser Cache API, Cloudflare Workers do not support the `ignoreSearch` or `ignoreVary` options on `get()`. You can accomplish this behavior by removing query strings or HTTP headers at `put()` time.
    /// In addition, the `stale-while-revalidate` and `stale-if-error` directives are not supported
    /// when using the `cache.put` or `cache.get` methods.
    ///
    /// Our implementation of the Cache API respects the following HTTP headers on the request passed to `get()`:
    ///
    /// - Range
    ///   - Results in a `206` response if a matching response with a Content-Length header is found. Your Cloudflare cache always respects range requests, even if an `Accept-Ranges` header is on the response.
    /// - If-Modified-Since
    ///   - Results in a `304` response if a matching response is found with a `Last-Modified` header with a value after the time specified in `If-Modified-Since`.
    /// - If-None-Match
    ///   - Results in a `304` response if a matching response is found with an `ETag` header with a value that matches a value in `If-None-Match.`
    pub async fn get<K: Into<CacheKey>>(
        &self,
        key: K,
        ignore_method: bool,
    ) -> Result<Option<http::Response<Body>>> {
        let fut = {
            let mut options = web_sys::CacheQueryOptions::new();
            options.ignore_method(ignore_method);

            let promise = match key.into() {
                CacheKey::Url(url) => self
                    .inner
                    .match_with_str_and_options(url.as_str(), &options),
                CacheKey::Request(req) => self.inner.match_with_request_and_options(&req, &options),
            };

            SendJsFuture::from(promise)
        };

        // `match` returns either a response or undefined
        let result = fut.await?;
        if result.is_undefined() {
            Ok(None)
        } else {
            let edge_response: web_sys::Response = result.into();
            let response = response::from_wasm(edge_response);
            Ok(Some(response))
        }
    }

    /// Deletes the [`Response`] object associated with the key.
    ///
    /// Returns:
    ///   - Success, if the response was cached but is now deleted
    ///   - ResponseNotFound, if the response was not in the cache at the time of deletion
    pub async fn delete<K: Into<CacheKey>>(
        &self,
        key: K,
        ignore_method: bool,
    ) -> Result<CacheDeletionOutcome> {
        let fut = {
            let mut options = web_sys::CacheQueryOptions::new();
            options.ignore_method(ignore_method);

            let promise = match key.into() {
                CacheKey::Url(url) => self
                    .inner
                    .delete_with_str_and_options(url.as_str(), &options),
                CacheKey::Request(req) => {
                    self.inner.delete_with_request_and_options(&req, &options)
                }
            };

            SendJsFuture::from(promise)
        };

        let result = fut.await?;
        // Unwrap is safe because we know this is a boolean
        // https://developers.cloudflare.com/workers/runtime-apis/cache#delete
        if result.as_bool().unwrap() {
            Ok(CacheDeletionOutcome::Success)
        } else {
            Ok(CacheDeletionOutcome::ResponseNotFound)
        }
    }
}

/// The `String` or `Request` object used as the lookup key. `String`s are interpreted as the URL for a new `Request` object.
pub enum CacheKey {
    Url(String),
    Request(web_sys::Request),
}

impl From<&str> for CacheKey {
    fn from(url: &str) -> Self {
        Self::Url(url.to_string())
    }
}

impl From<String> for CacheKey {
    fn from(url: String) -> Self {
        Self::Url(url)
    }
}

impl From<&String> for CacheKey {
    fn from(url: &String) -> Self {
        Self::Url(url.clone())
    }
}

impl From<web_sys::Request> for CacheKey {
    fn from(req: web_sys::Request) -> Self {
        Self::Request(req)
    }
}

impl From<http::Request<Body>> for CacheKey {
    fn from(req: http::Request<Body>) -> Self {
        Self::Request(request::into_wasm(req))
    }
}

pub struct CacheValue(web_sys::Response);

impl From<web_sys::Response> for CacheValue {
    fn from(res: web_sys::Response) -> Self {
        Self(res)
    }
}

impl From<http::Response<Body>> for CacheValue {
    fn from(res: http::Response<Body>) -> Self {
        Self(response::into_wasm(res))
    }
}

/// Successful outcomes when attempting to delete a `Response` from the cache
#[derive(Serialize)]
pub enum CacheDeletionOutcome {
    /// The response was cached but is now deleted
    Success,
    /// The response was not in the cache at the time of deletion.
    ResponseNotFound,
}
