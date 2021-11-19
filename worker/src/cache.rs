use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use worker_sys::cache::Cache as EdgeCache;
use worker_sys::Response as EdgeResponse;
use worker_sys::WorkerGlobalScope;

use crate::request::Request;
use crate::response::Response;
use crate::Result;

/// Provides access to the [cache api](https://developers.cloudflare.com/workers/runtime-apis/cache).
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
pub struct Cache {
    inner: EdgeCache,
}

impl Default for Cache {
    fn default() -> Self {
        let global: WorkerGlobalScope = js_sys::global().unchecked_into();

        Self {
            inner: global.caches().default(),
        }
    }
}

impl Cache {
    /// Opens a [`Cache`] by name. To access the default global cache, use [`Cache::default()`](`Default::default`).
    pub async fn open(name: String) -> Self {
        let global: WorkerGlobalScope = js_sys::global().unchecked_into();
        let cache = global.caches().open(name);

        // unwrap is safe because this promise never rejects
        // https://developer.mozilla.org/en-US/docs/Web/API/CacheStorage/open
        let inner = JsFuture::from(cache).await.unwrap().into();

        Self { inner }
    }

    /// Adds to the cache a [`Response`] keyed to the given request.
    ///
    /// The `stale-while-revalidate` and `stale-if-error` directives are not supported
    /// when using the `cache.put` or `cache.get` methods.
    ///
    /// Will throw an error if:
    /// - the request passed is a method other than GET.
    /// - the response passed has a status of 206 Partial Content.
    /// - the response passed contains the header `Vary: *` (required by the Cache API specification).
    pub async fn put<'a, K: Into<CacheKey<'a>>>(&self, key: K, response: Response) -> Result<()> {
        let promise = match key.into() {
            CacheKey::Url(url) => self.inner.put_url(url, response.into()),
            CacheKey::Request(request) => {
                // TODO: use from?
                let ffi_request = request.inner().clone()?;
                self.inner.put_request(ffi_request, response.into())
            }
        };
        let _ = JsFuture::from(promise).await?;
        Ok(())
    }

    /// Returns the [`Response`] object keyed to that request. Never sends a subrequest to the origin. If no matching response is found in cache, returns `None`.
    ///
    /// Unlike the browser Cache API, Cloudflare Workers do not support the `ignoreSearch` or `ignoreVary` options on `get()`. You can accomplish this behavior by removing query strings or HTTP headers at `put()` time.
    ///
    /// Our implementation of the Cache API respects the following HTTP headers on the request passed to `get()`:
    ///
    /// - Range
    ///   - Results in a `206` response if a matching response with a Content-Length header is found. Your Cloudflare cache always respects range requests, even if an `Accept-Ranges` header is on the response.
    /// - If-Modified-Since
    ///   - Results in a `304` response if a matching response is found with a `Last-Modified` header with a value after the time specified in `If-Modified-Since`.
    /// - If-None-Match
    ///   - Results in a `304` response if a matching response is found with an `ETag` header with a value that matches a value in `If-None-Match.`
    pub async fn get<'a, K: Into<CacheKey<'a>>>(
        &self,
        key: K,
        ignore_method: bool,
    ) -> Result<Option<Response>> {
        let options = JsValue::from_serde(&MatchOptions { ignore_method })?;
        let promise = match key.into() {
            CacheKey::Url(url) => self.inner.match_url(url, options),
            CacheKey::Request(request) => {
                // TODO: the same thing as above, why can't i use Into::into()?
                let ffi_request = request.inner().clone()?;
                self.inner.match_request(ffi_request, options)
            }
        };

        // `match` returns either a response or undefined
        let result = JsFuture::from(promise).await?;
        if result.is_undefined() {
            Ok(None)
        } else {
            let edge_response: EdgeResponse = result.into();
            let response = Response::from(edge_response);
            Ok(Some(response))
        }
    }

    pub async fn delete<'a, K: Into<CacheKey<'a>>>(
        &self,
        key: K,
        ignore_method: bool,
    ) -> Result<CacheDeletionOutcome> {
        let options = JsValue::from_serde(&MatchOptions { ignore_method })?;

        let promise = match key.into() {
            CacheKey::Url(url) => self.inner.delete_url(url, options),
            CacheKey::Request(request) => {
                // TODO: the same thing as above, why can't i use Into::into()?
                let ffi_request = request.inner().clone()?;
                self.inner.delete_request(ffi_request, options)
            }
        };
        let result = JsFuture::from(promise).await?;

        // unwrap is safe because we know this is a boolean
        // https://developers.cloudflare.com/workers/runtime-apis/cache#delete
        if result.as_bool().unwrap() {
            Ok(CacheDeletionOutcome::Success)
        } else {
            Ok(CacheDeletionOutcome::ResponseNotFound)
        }
    }
}

/// Can contain one possible property: `ignoreMethod`
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatchOptions {
    /// Consider the request method a `GET` regardless of its actual value.
    ignore_method: bool,
}

/// The `String` or `Request` object used as the lookup key. `String`s are interpreted as the URL for a new `Request` object.
pub enum CacheKey<'a> {
    Url(String),
    Request(&'a Request),
}

impl<S: Into<String>> From<S> for CacheKey<'_> {
    fn from(url: S) -> Self {
        Self::Url(url.into())
    }
}

impl<'a> From<&'a Request> for CacheKey<'a> {
    fn from(request: &'a Request) -> Self {
        Self::Request(request)
    }
}

/// Successful outcomes when attempting to delete a `Response` from the cache
pub enum CacheDeletionOutcome {
    /// The response was cached but is now deleted
    Success,
    /// The response was not in the cache at the time of deletion.
    ResponseNotFound,
}
