use std::collections::HashMap;

use crate::headers::Headers;
use crate::http::Method;

use js_sys::{self, Object};
use serde::Serialize;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

/// Optional options struct that contains settings to apply to the `Request`.
#[derive(Debug)]
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
    /// The cache mode for the request. `None` means use the default behavior.
    pub cache: Option<CacheMode>,
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

    pub fn with_cache(&mut self, cache: CacheMode) -> &mut Self {
        self.cache = Some(cache);
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
        let inner = web_sys::RequestInit::new();

        if !req.headers.is_empty() {
            inner.set_headers(req.headers.as_ref());
        }

        inner.set_method(req.method.as_ref());
        inner.set_redirect(req.redirect.into());
        if let Some(cache) = req.cache {
            inner.set_cache(cache.into());
        }
        if let Some(body) = req.body.as_ref() {
            inner.set_body(body);
        }

        // set the Cloudflare-specific `cf` property on FFI RequestInit
        if !req.cf.is_default() {
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
    }

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
            cache: None,
        }
    }
}

/// <https://developers.cloudflare.com/workers/runtime-apis/request#requestinitcfproperties>
#[derive(Debug)]
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
    pub cache_ttl: Option<i32>,
    /// This option is a version of the cacheTtl feature which chooses a TTL based on the response’s
    /// status code. If the response to this request has a status code that matches, Cloudflare will
    /// cache for the instructed time, and override cache directives sent by the origin. For
    /// example: { "200-299": 86400, 404: 1, "500-599": 0 }. The value can be any integer, including
    /// zero and negative integers. A value of 0 indicates that the cache asset expires immediately.
    /// Any negative value instructs Cloudflare not to cache at all.
    pub cache_ttl_by_status: Option<HashMap<String, i32>>,
    /// Enables Image Resizing for this request.
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
            &JsValue::from(props.minify.unwrap_or(defaults.minify.unwrap_or_default())),
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

    pub fn is_default(&self) -> bool {
        let de = CfProperties::default();
        self.apps == de.apps
            && self.cache_everything == de.cache_everything
            && self.cache_key == de.cache_key
            && self.cache_ttl == de.cache_ttl
            && self.cache_ttl_by_status == de.cache_ttl_by_status
            && self.minify == de.minify
            && self.mirage == de.mirage
            && self.image.is_none()
            && self.polish == de.polish
            && self.resolve_override == de.resolve_override
            && self.scrape_shield == de.scrape_shield
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
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct MinifyConfig {
    pub js: bool,
    pub html: bool,
    pub css: bool,
}

/// Configuration options for Cloudflare's image optimization feature:
/// <https://blog.cloudflare.com/introducing-polish-automatic-image-optimizati/>
#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PolishConfig {
    #[default]
    Off,
    Lossy,
    Lossless,
}

#[derive(Clone, Debug, serde::Serialize)]
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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeOriginAuth {
    SharePublicly,
}

/// Configuration options for Cloudflare's image resizing feature:
/// <https://developers.cloudflare.com/images/image-resizing/>
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResizeConfig {
    /// Whether to preserve animation frames from input files. Default is `true`. Setting it to `false` reduces animations to still images.
    pub anim: Option<bool>,
    /// Background color to add underneath the image. Applies to images with transparency (for example, PNG) and images resized with `fit=pad`.
    pub background: Option<String>,
    /// Blur radius between `1` (slight blur) and `250` (maximum). Be aware that you cannot use this option to reliably obscure image content.
    pub blur: Option<u8>,
    /// Adds a border around the image. The border is added after resizing.
    pub border: Option<ResizeBorder>,
    /// Increase brightness by a factor. A value of `1.0` equals no change, a value of `0.5` equals half brightness, and a value of `2.0` equals twice as bright.
    pub brightness: Option<f64>,
    /// Slightly reduces latency on a cache miss by selecting a quickest-to-compress file format, at a cost of increased file size and lower image quality.
    pub compression: Option<ResizeCompression>,
    /// Increase contrast by a factor. A value of `1.0` equals no change, a value of `0.5` equals low contrast, and a value of `2.0` equals high contrast.
    pub contrast: Option<f64>,
    /// Device Pixel Ratio. Default is `1`. Multiplier for `width`/`height` that makes it easier to specify higher-DPI sizes in `<img srcset>`.
    pub dpr: Option<f64>,
    /// Drawing operations to overlay on the image.
    pub draw: Option<ResizeDraw>,
    /// Affects interpretation of `width` and `height`. All resizing modes preserve aspect ratio.
    pub fit: Option<ResizeFit>,
    /// Flips the image horizontally, vertically, or both. Can be used with the `rotate` parameter to set the orientation of an image.
    pub flip: Option<ResizeFlip>,
    /// The `auto` option will serve the WebP or AVIF format to browsers that support it. If this option is not specified, a standard format like JPEG or PNG will be used.
    pub format: Option<ResizeFormat>,
    /// Increase exposure by a factor. A value of `1.0` equals no change, a value of `0.5` darkens the image, and a value of `2.0` lightens the image.
    pub gamma: Option<f64>,
    /// Specifies how an image should be cropped when used with `fit=cover` and `fit=crop`. Available options are `auto`, `face`, a side (`left`, `right`, `top`, `bottom`), and relative coordinates (`XxY` with a valid range of `0.0` to `1.0`).
    pub gravity: Option<ResizeGravity>,
    /// Specifies maximum height of the image in pixels. Exact behavior depends on the `fit` mode (described below).
    pub height: Option<usize>,
    /// Controls amount of invisible metadata (EXIF data) that should be preserved. Color profiles and EXIF rotation are applied to the image even if the metadata is discarded.
    pub metadata: Option<ResizeMetadata>,
    /// Authentication method for accessing the origin image.
    pub origin_auth: Option<ResizeOriginAuth>,
    /// In case of a fatal error that prevents the image from being resized, redirects to the unresized source image URL. This may be useful in case some images require user authentication and cannot be fetched anonymously via Worker.
    pub onerror: Option<ResizeOnerror>,
    /// Specifies quality for images in JPEG, WebP, and AVIF formats. The quality is in a 1-100 scale, but useful values are between `50` (low quality, small file size) and `90` (high quality, large file size).
    pub quality: Option<ResizeQuality>,
    /// Number of degrees (`90`, `180`, or `270`) to rotate the image by. `width` and `height` options refer to axes after rotation.
    pub rotate: Option<usize>,
    /// Increases saturation by a factor. A value of `1.0` equals no change, a value of `0.5` equals half saturation, and a value of `2.0` equals twice as saturated.
    pub saturation: Option<f64>,
    /// Specifies strength of sharpening filter to apply to the image. The value is a floating-point number between `0` (no sharpening, default) and `10` (maximum).
    pub sharpen: Option<f64>,
    /// Specifies a number of pixels to cut off on each side. Allows removal of borders or cutting out a specific fragment of an image.
    pub trim: Option<ResizeTrim>,
    /// Specifies maximum width of the image. Exact behavior depends on the `fit` mode; use the `fit=scale-down` option to ensure that the image will not be enlarged unnecessarily.
    pub width: Option<usize>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeCompression {
    Fast,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum ResizeDrawRepeat {
    Uniform(bool),
    Axis(String),
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ResizeDraw {
    url: String,
    opacity: Option<f64>,
    repeat: Option<ResizeDrawRepeat>,
    top: Option<usize>,
    bottom: Option<usize>,
    left: Option<usize>,
    right: Option<usize>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeFit {
    ScaleDown,
    Contain,
    Cover,
    Crop,
    Pad,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum ResizeFlip {
    #[serde(rename = "h")]
    Horizontally,
    #[serde(rename = "v")]
    Vertically,
    #[serde(rename = "hv")]
    Both,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeFormat {
    Avif,
    Webp,
    Json,
    Jpeg,
    Png,
    BaselineJpeg,
    PngForce,
    Svg,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeGravitySide {
    Auto,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum ResizeGravity {
    Side(ResizeGravitySide),
    Coords { x: f64, y: f64 },
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeQualityLiteral {
    Low,
    MediumLow,
    MediumHigh,
    High,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
#[serde(untagged)]
pub enum ResizeQuality {
    Literal(ResizeQualityLiteral),
    Specific { value: usize },
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeMetadata {
    Keep,
    Copyright,
    None,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResizeOnerror {
    Redirect,
}

#[derive(Clone, Copy, Debug, Serialize)]
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

#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequestRedirect {
    Error,
    #[default]
    Follow,
    Manual,
}

impl From<RequestRedirect> for &str {
    fn from(redirect: RequestRedirect) -> Self {
        match redirect {
            RequestRedirect::Error => "error",
            RequestRedirect::Follow => "follow",
            RequestRedirect::Manual => "manual",
        }
    }
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

/// Cache mode for controlling how requests interact with the cache.
/// Corresponds to JavaScript's `RequestInit.cache` property.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheMode {
    NoStore,
    NoCache,
    Reload,
}

impl From<CacheMode> for web_sys::RequestCache {
    fn from(mode: CacheMode) -> Self {
        match mode {
            CacheMode::NoStore => web_sys::RequestCache::NoStore,
            CacheMode::NoCache => web_sys::RequestCache::NoCache,
            CacheMode::Reload => web_sys::RequestCache::Reload,
        }
    }
}

#[test]
fn request_init_no_invalid_options() {
    let mut init = RequestInit::new();
    init.method = Method::Post;

    let js_init: web_sys::RequestInit = (&init).into();

    let _ = web_sys::Request::new_with_str_and_init(
        "https://httpbin.org/post",
        &js_init,
    ).unwrap();
}
