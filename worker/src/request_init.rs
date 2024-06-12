use std::collections::HashMap;

use crate::headers::Headers;
use crate::http::Method;

use js_sys::{self, Object};
use serde::Serialize;
use wasm_bindgen::JsValue;

/// Optional options struct that contains settings to apply to the `Request`.
pub struct RequestInit {
    /// Currently requires a manual conversion from your data into a [`wasm_bindgen::JsValue`].
    pub body: Option<JsValue>,
    /// Headers associated with the outbound `Request`.
    pub headers: Headers,
    /// Cloudflare-specific properties that can be set on the `Request` that control how Cloudflare’s
    /// edge handles the request.
    pub cf: CfProperties,
    /// The HTTP Method used for this `Request`.
    pub method: Method,
    /// The redirect mode to use: follow, error, or manual. The default for a new Request object is
    /// follow. Note, however, that the incoming Request property of a FetchEvent will have redirect
    /// mode manual.
    pub redirect: RequestRedirect,
}

impl RequestInit {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_headers(&mut self, headers: Headers) -> &mut Self {
        self.headers = headers;
        self
    }

    pub fn with_method(&mut self, method: Method) -> &mut Self {
        self.method = method;
        self
    }

    pub fn with_redirect(&mut self, redirect: RequestRedirect) -> &mut Self {
        self.redirect = redirect;
        self
    }

    pub fn with_body(&mut self, body: Option<JsValue>) -> &mut Self {
        self.body = body;
        self
    }

    pub fn with_cf_properties(&mut self, props: CfProperties) -> &mut Self {
        self.cf = props;
        self
    }
}

impl From<&RequestInit> for web_sys::RequestInit {
    fn from(req: &RequestInit) -> Self {
        let mut inner = web_sys::RequestInit::new();
        inner.headers(req.headers.as_ref());
        inner.method(req.method.as_ref());
        inner.redirect(req.redirect.into());
        inner.body(req.body.as_ref());

        // set the Cloudflare-specific `cf` property on FFI RequestInit
        let r = ::js_sys::Reflect::set(
            inner.as_ref(),
            &JsValue::from("cf"),
            &JsValue::from(&req.cf),
        );
        debug_assert!(
            r.is_ok(),
            "setting properties should never fail on our dictionary objects"
        );
        let _ = r;

        inner
    }
}

impl Default for RequestInit {
    fn default() -> Self {
        Self {
            body: None,
            headers: Headers::new(),
            cf: CfProperties::default(),
            method: Method::Get,
            redirect: RequestRedirect::default(),
        }
    }
}

/// <https://developers.cloudflare.com/workers/runtime-apis/request#requestinitcfproperties>
pub struct CfProperties {
    /// Whether Cloudflare Apps should be enabled for this request. Defaults to `true`.
    pub apps: Option<bool>,
    /// This option forces Cloudflare to cache the response for this request, regardless of what
    /// headers are seen on the response. This is equivalent to setting the page rule “Cache Level”
    /// (to “Cache Everything”). Defaults to `false`.
    pub cache_everything: Option<bool>,
    /// A request’s cache key is what determines if two requests are “the same” for caching
    /// purposes. If a request has the same cache key as some previous request, then we can serve
    /// the same cached response for both.
    pub cache_key: Option<String>,
    /// This option forces Cloudflare to cache the response for this request, regardless of what
    /// headers are seen on the response. This is equivalent to setting two page rules: “Edge Cache
    /// TTL” and “Cache Level” (to “Cache Everything”). The value must be zero or a positive number.
    /// A value of 0 indicates that the cache asset expires immediately.
    pub cache_ttl: Option<u32>,
    /// This option is a version of the cacheTtl feature which chooses a TTL based on the response’s
    /// status code. If the response to this request has a status code that matches, Cloudflare will
    /// cache for the instructed time, and override cache directives sent by the origin. For
    /// example: { "200-299": 86400, 404: 1, "500-599": 0 }. The value can be any integer, including
    /// zero and negative integers. A value of 0 indicates that the cache asset expires immediately.
    /// Any negative value instructs Cloudflare not to cache at all.
    pub cache_ttl_by_status: Option<HashMap<String, i32>>,
    // TODO docs
    pub image: Option<ResizeConfig>,
    /// Enables or disables AutoMinify for various file types.
    /// For example: `{ javascript: true, css: true, html: false }`.
    pub minify: Option<MinifyConfig>,
    /// Whether Mirage should be enabled for this request, if otherwise configured for this zone.
    /// Defaults to true.
    pub mirage: Option<bool>,
    /// Sets Polish mode. The possible values are lossy, lossless or off.
    pub polish: Option<PolishConfig>,
    /// Directs the request to an alternate origin server by overriding the DNS lookup. The value of
    /// `resolve_override` specifies an alternate hostname which will be used when determining the
    /// origin IP address, instead of using the hostname specified in the URL. The Host header of
    /// the request will still match what is in the URL. Thus, `resolve_override` allows a request  
    /// to be sent to a different server than the URL / Host header specifies. However,
    /// `resolve_override` will only take effect if both the URL host and the host specified by
    /// `resolve_override` are within your zone. If either specifies a host from a different zone /
    /// domain, then the option will be ignored for security reasons. If you need to direct a
    /// request to a host outside your zone (while keeping the Host header pointing within your
    /// zone), first create a CNAME record within your zone pointing to the outside host, and then
    /// set `resolve_override` to point at the CNAME record.
    ///
    /// Note that, for security reasons, it is not possible to set the Host header to specify a host
    /// outside of your zone unless the request is actually being sent to that host.
    pub resolve_override: Option<String>,
    /// Whether ScrapeShield should be enabled for this request, if otherwise configured for this
    /// zone. Defaults to `true`.
    pub scrape_shield: Option<bool>,
}

impl From<&CfProperties> for JsValue {
    fn from(props: &CfProperties) -> Self {
        let obj = js_sys::Object::new();
        let defaults = CfProperties::default();
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);

        set_prop(
            &obj,
            &JsValue::from("apps"),
            &JsValue::from(props.apps.unwrap_or(defaults.apps.unwrap_or_default())),
        );

        set_prop(
            &obj,
            &JsValue::from("cacheEverything"),
            &JsValue::from(
                props
                    .cache_everything
                    .unwrap_or(defaults.cache_everything.unwrap_or_default()),
            ),
        );

        set_prop(
            &obj,
            &JsValue::from("cacheKey"),
            &JsValue::from(
                props
                    .cache_key
                    .clone()
                    .unwrap_or(defaults.cache_key.unwrap_or_default()),
            ),
        );

        set_prop(
            &obj,
            &JsValue::from("cacheTtl"),
            &JsValue::from(
                props
                    .cache_ttl
                    .unwrap_or(defaults.cache_ttl.unwrap_or_default()),
            ),
        );

        let ttl_status_map = props
            .cache_ttl_by_status
            .clone()
            .unwrap_or(defaults.cache_ttl_by_status.unwrap_or_default());
        set_prop(
            &obj,
            &JsValue::from("cacheTtlByStatus"),
            &ttl_status_map.serialize(&serializer).unwrap_or_default(),
        );

        set_prop(
            &obj,
            &JsValue::from("minify"),
            &serde_wasm_bindgen::to_value(
                &props.minify.unwrap_or(defaults.minify.unwrap_or_default()),
            )
            .unwrap(),
        );

        set_prop(
            &obj,
            &JsValue::from("mirage"),
            &JsValue::from(props.mirage.unwrap_or(defaults.mirage.unwrap_or_default())),
        );

        let polish_val = props.polish.unwrap_or(defaults.polish.unwrap_or_default());
        set_prop(
            &obj,
            &JsValue::from("polish"),
            &serde_wasm_bindgen::to_value(&polish_val).unwrap(),
        );

        set_prop(
            &obj,
            &JsValue::from("resolveOverride"),
            &JsValue::from(
                props
                    .resolve_override
                    .clone()
                    .unwrap_or(defaults.resolve_override.unwrap_or_default()),
            ),
        );

        set_prop(
            &obj,
            &JsValue::from("scrapeShield"),
            &JsValue::from(
                props
                    .scrape_shield
                    .unwrap_or(defaults.scrape_shield.unwrap_or_default()),
            ),
        );

        if let Some(image) = &props.image {
            set_prop(
                &obj,
                &JsValue::from("image"),
                &serde_wasm_bindgen::to_value(&image).unwrap(),
            );
        }

        obj.into()
    }
}

fn set_prop(target: &Object, key: &JsValue, val: &JsValue) {
    let r = ::js_sys::Reflect::set(target, key, val);
    debug_assert!(
        r.is_ok(),
        "setting properties should never fail on our dictionary objects"
    );
    let _ = r;
}

impl CfProperties {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for CfProperties {
    fn default() -> Self {
        Self {
            apps: Some(true),
            cache_everything: Some(false),
            cache_key: None,
            cache_ttl: None,
            cache_ttl_by_status: None,
            minify: None,
            mirage: Some(true),
            image: None,
            polish: None,
            resolve_override: None,
            scrape_shield: Some(true),
        }
    }
}

/// Configuration options for Cloudflare's minification features:
/// <https://www.cloudflare.com/website-optimization/>
#[derive(Clone, Copy, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct MinifyConfig {
    pub js: bool,
    pub html: bool,
    pub css: bool,
}

/// Configuration options for Cloudflare's image optimization feature:
/// <https://blog.cloudflare.com/introducing-polish-automatic-image-optimizati/>
#[derive(Clone, Copy, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolishConfig {
    #[default]
    Off,
    Lossy,
    Lossless,
}

#[derive(Clone, serde::Serialize)]
#[serde(untagged)]
pub enum ResizeBorder {
    Uniform {
        color: String,
        width: usize,
    },
    Varying {
        color: String,
        top: usize,
        right: usize,
        bottom: usize,
        left: usize,
    },
}

/// Configuration options for Cloudflare's image resizing feature:
/// <https://developers.cloudflare.com/images/image-resizing/>
#[derive(Clone, Default, serde::Serialize)]
pub struct ResizeConfig {
    pub anim: Option<bool>,
    pub background: Option<String>,
    pub blur: Option<u8>,
    pub border: Option<ResizeBorder>,
    pub brightness: Option<f64>,
    pub compression: Option<ResizeCompression>,
    pub contrast: Option<f64>,
    pub dpr: Option<f64>,
    pub fit: Option<ResizeFit>,
    pub format: Option<ResizeFormat>,
    pub gamma: Option<f64>,
    pub gravity: Option<ResizeGravity>,
    pub height: Option<usize>,
    pub metadata: Option<ResizeMetadata>,
    pub onerror: Option<ResizeOnerror>,
    pub quality: Option<usize>,
    pub rotate: Option<usize>,
    pub sharpen: Option<usize>,
    pub trim: Option<ResizeTrim>,
    pub width: Option<usize>,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeCompression {
    Fast,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeFit {
    ScaleDown,
    Contain,
    Cover,
    Crop,
    Pad,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeFormat {
    Auto,
    Avif,
    Webp,
    Json,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeGravitySide {
    Auto,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum ResizeGravity {
    Side(ResizeGravitySide),
    Coords { x: f64, y: f64 },
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeMetadata {
    Keep,
    Copyright,
    None,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeOnerror {
    Redirect,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResizeTrim {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<usize>,
}

#[derive(Clone, Copy, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequestRedirect {
    Error,
    #[default]
    Follow,
    Manual,
}

impl From<RequestRedirect> for web_sys::RequestRedirect {
    fn from(redir: RequestRedirect) -> Self {
        match redir {
            RequestRedirect::Error => web_sys::RequestRedirect::Error,
            RequestRedirect::Follow => web_sys::RequestRedirect::Follow,
            RequestRedirect::Manual => web_sys::RequestRedirect::Manual,
        }
    }
}
