use std::collections::HashMap;

use crate::headers::Headers;
use crate::http::Method;

use js_sys::{self, Object};
use wasm_bindgen::{prelude::*, JsValue};

pub struct RequestInit {
    pub body: Option<JsValue>,
    pub headers: Headers,
    pub cf: CfProperties,
    pub method: Method,
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

impl From<&RequestInit> for edgeworker_ffi::RequestInit {
    fn from(req: &RequestInit) -> Self {
        let mut inner = edgeworker_ffi::RequestInit::new();
        inner.headers(req.headers.as_ref());
        inner.method(&req.method.to_string());
        inner.redirect(req.redirect.clone().into());
        inner.body(req.body.as_ref());

        // set the Cloudflare-specific `cf` property on FFI RequestInit
        #[allow(unused_unsafe)]
        let r = unsafe {
            ::js_sys::Reflect::set(
                inner.as_ref(),
                &JsValue::from("cf"),
                &JsValue::from(&req.cf),
            )
        };
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
        Self::new()
    }
}

// https://developers.cloudflare.com/workers/runtime-apis/request#requestinitcfproperties
pub struct CfProperties {
    pub apps: Option<bool>,
    pub cache_everything: Option<bool>,
    pub cache_key: Option<bool>,
    pub cache_ttl: Option<u32>,
    pub cache_ttl_by_status: Option<HashMap<String, u32>>,
    pub minify: Option<MinifyConfig>,
    pub mirage: Option<bool>,
    pub polish: Option<PolishConfig>,
    pub resolve_override: Option<String>,
    pub scrape_shield: Option<bool>,
}

impl From<&CfProperties> for JsValue {
    fn from(props: &CfProperties) -> Self {
        let obj = js_sys::Object::new();
        let defaults = CfProperties::default();

        set_prop(
            &obj,
            &JsValue::from("apps"),
            &JsValue::from(props.apps.unwrap_or(defaults.apps.unwrap())),
        );

        set_prop(
            &obj,
            &JsValue::from("cacheEverything"),
            &JsValue::from(
                props
                    .cache_everything
                    .unwrap_or(defaults.cache_everything.unwrap()),
            ),
        );

        set_prop(
            &obj,
            &JsValue::from("cacheKey"),
            &JsValue::from(props.cache_key.unwrap_or(defaults.cache_key.unwrap())),
        );

        set_prop(
            &obj,
            &JsValue::from("cacheTtl"),
            &JsValue::from(props.cache_ttl.unwrap_or(defaults.cache_ttl.unwrap())),
        );

        let ttl_status_map = props
            .cache_ttl_by_status
            .clone()
            .unwrap_or(defaults.cache_ttl_by_status.unwrap());
        set_prop(
            &obj,
            &JsValue::from("cacheTtlByStatus"),
            &JsValue::from_serde(&ttl_status_map).unwrap(),
        );

        set_prop(
            &obj,
            &JsValue::from("minify"),
            &JsValue::from(props.minify.unwrap_or(defaults.minify.unwrap())),
        );

        set_prop(
            &obj,
            &JsValue::from("mirage"),
            &JsValue::from(props.mirage.unwrap_or(defaults.mirage.unwrap())),
        );

        let polish_val: &str = props
            .polish
            .clone()
            .unwrap_or(defaults.polish.unwrap())
            .into();
        set_prop(&obj, &JsValue::from("polish"), &JsValue::from(polish_val));

        set_prop(
            &obj,
            &JsValue::from("resolveOverride"),
            &JsValue::from(
                props
                    .resolve_override
                    .clone()
                    .unwrap_or(defaults.resolve_override.unwrap()),
            ),
        );

        set_prop(
            &obj,
            &JsValue::from("scrapeShield"),
            &JsValue::from(
                props
                    .scrape_shield
                    .unwrap_or(defaults.scrape_shield.unwrap()),
            ),
        );

        obj.into()
    }
}

fn set_prop(target: &Object, key: &JsValue, val: &JsValue) {
    #[allow(unused_unsafe)]
    let r = unsafe { ::js_sys::Reflect::set(&target, key, val) };
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
            polish: None,
            resolve_override: None,
            scrape_shield: Some(true),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct MinifyConfig {
    pub js: bool,
    pub html: bool,
    pub css: bool,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum PolishConfig {
    Off,
    Lossy,
    Lossless,
}

impl From<PolishConfig> for &str {
    fn from(conf: PolishConfig) -> Self {
        match conf {
            PolishConfig::Off => "off",
            PolishConfig::Lossy => "lossy",
            PolishConfig::Lossless => "lossless",
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum RequestRedirect {
    Error,
    Follow,
    Manual,
}

impl Default for RequestRedirect {
    fn default() -> Self {
        RequestRedirect::Follow
    }
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

impl From<RequestRedirect> for edgeworker_ffi::RequestRedirect {
    fn from(redir: RequestRedirect) -> Self {
        match redir {
            RequestRedirect::Error => edgeworker_ffi::RequestRedirect::Error,
            RequestRedirect::Follow => edgeworker_ffi::RequestRedirect::Follow,
            RequestRedirect::Manual => edgeworker_ffi::RequestRedirect::Manual,
        }
    }
}
