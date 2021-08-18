use std::collections::HashMap;

use js_sys;
use wasm_bindgen::JsValue;

// https://developers.cloudflare.com/workers/runtime-apis/request#requestinitcfproperties
pub struct CfProperties {
    apps: Option<bool>,
    cache_everything: Option<bool>,
    cache_key: Option<bool>,
    cache_ttl: Option<u32>,
    cache_ttl_by_status: Option<HashMap<String, u32>>,
    minify: Option<MinifyConfig>,
    polish: Option<PolishConfig>,
    resolve_override: Option<String>,
    scrape_shield: Option<bool>,
}

impl From<&CfProperties> for JsValue {
    fn from(props: &CfProperties) -> Self {
        let obj = js_sys::Object::new();
        if let Some(val) = props.apps {
            #[allow(unused_unsafe)]
            let r = unsafe {
                ::js_sys::Reflect::set(&obj, &JsValue::from("apps"), &JsValue::from(val))
            };
            debug_assert!(
                r.is_ok(),
                "setting properties should never fail on our dictionary objects"
            );
            let _ = r;
        };

        obj.into()
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
            polish: None,
            resolve_override: None,
            scrape_shield: Some(true),
        }
    }
}

pub struct MinifyConfig {
    pub js: bool,
    pub html: bool,
    pub css: bool,
}

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

pub enum RequestRedirect {
    Error,
    Follow,
    Manual,
}

impl From<RequestRedirect> for &str {
    fn from(redirect: RequestRedirect) -> &'static str {
        match redirect {
            RequestRedirect::Error => "error",
            RequestRedirect::Follow => "follow",
            RequestRedirect::Manual => "manual",
        }
    }
}
