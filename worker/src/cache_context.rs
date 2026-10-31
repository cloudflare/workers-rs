use crate::{send::SendFuture, Error, Result};
use js_sys::futures::JsFuture;
use serde::{Deserialize, Serialize};
use worker_sys::CacheContext as JsCacheContext;

/// Invalidates responses stored by [Workers Cache](https://developers.cloudflare.com/workers/cache/).
///
/// Unlike [`crate::Cache`], this operates on the calling Worker entrypoint's cache,
/// not the Service Worker Cache API (`caches.default`). Obtain it from
/// [`crate::Context::cache()`] or [`cache()`].
#[derive(Debug, Clone)]
pub struct CacheContext {
    inner: JsCacheContext,
}

unsafe impl Send for CacheContext {}
unsafe impl Sync for CacheContext {}

impl CacheContext {
    /// Purges cached responses matching the supplied options.
    ///
    /// Tags and path prefixes may be combined; responses matching either are
    /// invalidated. Purging everything cannot be combined with either field.
    ///
    /// A resolved result may have `success: false` (for example, when rate limited).
    /// JavaScript exceptions and rejected promises are returned as [`Error`].
    pub async fn purge(&self, options: CachePurgeOptions) -> Result<CachePurgeResult> {
        if options.purge_everything && (options.tags.is_some() || options.path_prefixes.is_some()) {
            return Err(Error::RustError(
                "purgeEverything cannot be combined with tags or pathPrefixes".into(),
            ));
        }

        let promise = self.inner.purge(serde_wasm_bindgen::to_value(&options)?)?;
        let result = SendFuture::new(JsFuture::from(promise)).await?;
        Ok(serde_wasm_bindgen::from_value(result)?)
    }
}

impl From<JsCacheContext> for CacheContext {
    fn from(inner: JsCacheContext) -> Self {
        Self { inner }
    }
}

impl AsRef<JsCacheContext> for CacheContext {
    fn as_ref(&self) -> &JsCacheContext {
        &self.inner
    }
}

/// Accesses Workers Cache through the `cache` export of `cloudflare:workers`.
///
/// This requires a runtime providing that export. Use [`crate::Context::cache()`]
/// to check for availability through an execution context instead.
///
/// The exported proxy resolves the calling entrypoint when [`CacheContext::purge`]
/// is invoked. Purging still requires a context where Workers Cache is available.
///
/// ```no_run
/// # use worker::{CachePurgeOptions, Result};
/// # async fn example() -> Result<()> {
/// let result = worker::cache().purge(CachePurgeOptions::tags(["blog-posts"])).await?;
/// if !result.success {
///     worker::console_error!("Cache purge failed: {:?}", result.errors);
/// }
/// # Ok(())
/// # }
/// ```
pub fn cache() -> CacheContext {
    worker_sys::CACHE.with(|cache| cache.clone().into())
}

/// Selects responses to invalidate in the calling entrypoint's Workers Cache.
///
/// Set both `tags` and `path_prefixes` to invalidate their union. Setting either
/// alongside `purge_everything` is rejected by [`CacheContext::purge`].
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CachePurgeOptions {
    /// Values attached to cached responses with the `Cache-Tag` header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Request path prefixes, such as `/blog/`, without a hostname or query string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_prefixes: Option<Vec<String>>,
    /// Invalidate every cached response for the calling entrypoint.
    #[serde(skip_serializing_if = "is_false")]
    pub purge_everything: bool,
}

fn is_false(value: &bool) -> bool {
    !value
}

impl CachePurgeOptions {
    /// Invalidates responses tagged with any of the supplied values.
    pub fn tags<I, S>(tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            tags: Some(tags.into_iter().map(Into::into).collect()),
            ..Default::default()
        }
    }

    /// Invalidates responses whose request paths start with any supplied prefix.
    pub fn path_prefixes<I, S>(prefixes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            path_prefixes: Some(prefixes.into_iter().map(Into::into).collect()),
            ..Default::default()
        }
    }

    /// Invalidates every cached response for the calling entrypoint.
    pub fn everything() -> Self {
        Self {
            purge_everything: true,
            ..Default::default()
        }
    }
}

/// The runtime's response to a Workers Cache purge request.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CachePurgeResult {
    /// Whether the purge was accepted.
    pub success: bool,
    /// Errors explaining why the purge was not accepted.
    pub errors: Vec<CachePurgeError>,
}

/// A structured error returned by the Workers Cache purge service.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CachePurgeError {
    pub code: i32,
    pub message: String,
}

#[cfg(test)]
mod tests;
