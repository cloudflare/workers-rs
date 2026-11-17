use super::{CacheContext, CachePurgeOptions};
use crate::Context;
use serde_json::json;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen(inline_js = "
    export function cacheContextFixture(result, failure) {
        const fixture = {
            calls: 0,
            options: undefined,
            purge(options) {
                if (this !== fixture) throw new Error('incorrect cache receiver');
                this.calls++;
                this.options = options;
                if (failure === 'throw') throw new Error('synchronous purge failure');
                if (failure === 'reject') return Promise.reject(new Error('rejected purge'));
                return Promise.resolve(JSON.parse(result));
            }
        };
        return fixture;
    }
")]
extern "C" {
    #[wasm_bindgen(js_name = cacheContextFixture)]
    fn cache_context_fixture(result: &str, failure: &str) -> JsValue;
}

fn cache_context(fixture: &JsValue) -> CacheContext {
    CacheContext::from(fixture.clone().unchecked_into::<worker_sys::CacheContext>())
}

fn property(fixture: &JsValue, name: &str) -> JsValue {
    js_sys::Reflect::get(fixture, &name.into()).unwrap()
}

fn assert_send<T: Send>(value: T) -> T {
    value
}

#[wasm_bindgen_test]
fn options_use_runtime_property_names_and_omit_unused_fields() {
    let cases = [
        (CachePurgeOptions::default(), json!({})),
        (
            CachePurgeOptions::tags(["article", "product"]),
            json!({ "tags": ["article", "product"] }),
        ),
        (
            CachePurgeOptions::path_prefixes(["/assets/".to_string()]),
            json!({ "pathPrefixes": ["/assets/"] }),
        ),
        (
            CachePurgeOptions::everything(),
            json!({ "purgeEverything": true }),
        ),
    ];

    for (options, expected) in cases {
        let value = serde_wasm_bindgen::to_value(&options).unwrap();
        let actual: serde_json::Value = serde_wasm_bindgen::from_value(value).unwrap();
        assert_eq!(actual, expected);
    }
}

#[wasm_bindgen_test]
async fn purge_passes_combined_tags_and_prefixes_and_decodes_success() {
    let fixture = cache_context_fixture(r#"{"success":true,"errors":[]}"#, "");
    let options = CachePurgeOptions {
        tags: Some(vec!["article".to_string()]),
        path_prefixes: Some(vec!["/articles/".to_string()]),
        ..Default::default()
    };

    let cache = cache_context(&fixture);
    let result = assert_send(cache.purge(options)).await.unwrap();

    assert!(result.success);
    assert!(result.errors.is_empty());
    let actual: serde_json::Value =
        serde_wasm_bindgen::from_value(property(&fixture, "options")).unwrap();
    assert_eq!(
        actual,
        json!({ "tags": ["article"], "pathPrefixes": ["/articles/"] })
    );
}

#[wasm_bindgen_test]
async fn purge_preserves_structured_service_errors() {
    let fixture = cache_context_fixture(
        r#"{"success":false,"errors":[{"code":1001,"message":"Purge limit exceeded"}]}"#,
        "",
    );

    let result = cache_context(&fixture)
        .purge(CachePurgeOptions::everything())
        .await
        .unwrap();

    assert!(!result.success);
    assert_eq!(result.errors.len(), 1);
    assert_eq!(result.errors[0].code, 1001);
    assert_eq!(result.errors[0].message, "Purge limit exceeded");
}

#[wasm_bindgen_test]
async fn purge_rejects_runtime_results_missing_required_fields() {
    for (response, missing_field) in [
        (r#"{"errors":[]}"#, "success"),
        (r#"{"success":true}"#, "errors"),
    ] {
        let fixture = cache_context_fixture(response, "");

        let error = cache_context(&fixture)
            .purge(CachePurgeOptions::everything())
            .await
            .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("missing field"), "{message}");
        assert!(message.contains(missing_field), "{message}");
    }
}

#[wasm_bindgen_test]
async fn purge_rejects_incompatible_options_before_calling_runtime() {
    // Even an explicitly empty list is incompatible with purgeEverything.
    for (tags, path_prefixes) in [
        (Some(vec!["article".to_string()]), None),
        (Some(vec![]), None),
        (None, Some(vec!["/articles/".to_string()])),
        (None, Some(vec![])),
    ] {
        let fixture = cache_context_fixture(r#"{"success":true,"errors":[]}"#, "");
        let options = CachePurgeOptions {
            tags,
            path_prefixes,
            purge_everything: true,
        };

        assert!(cache_context(&fixture).purge(options).await.is_err());
        assert_eq!(property(&fixture, "calls").as_f64(), Some(0.0));
    }
}

#[wasm_bindgen_test]
async fn purge_propagates_synchronous_exceptions() {
    let fixture = cache_context_fixture("null", "throw");

    let error = cache_context(&fixture)
        .purge(CachePurgeOptions::tags(["article"]))
        .await
        .unwrap_err();

    assert!(error.to_string().contains("synchronous purge failure"));
}

#[wasm_bindgen_test]
async fn purge_propagates_rejected_promises() {
    let fixture = cache_context_fixture("null", "reject");

    let error = cache_context(&fixture)
        .purge(CachePurgeOptions::everything())
        .await
        .unwrap_err();

    assert!(error.to_string().contains("rejected purge"));
}

#[wasm_bindgen_test]
fn execution_context_cache_is_optional() {
    let object = js_sys::Object::new();
    let context = Context::new(object.clone().unchecked_into());
    assert!(context.cache().is_none());

    js_sys::Reflect::set(&object, &"cache".into(), &JsValue::NULL).unwrap();
    assert!(context.cache().is_none());
}

#[wasm_bindgen_test]
async fn execution_context_cache_preserves_the_method_receiver() {
    let fixture = cache_context_fixture(r#"{"success":true,"errors":[]}"#, "");
    let object = js_sys::Object::new();
    js_sys::Reflect::set(&object, &"cache".into(), &fixture).unwrap();
    let context = Context::new(object.unchecked_into());

    let result = context
        .cache()
        .unwrap()
        .purge(CachePurgeOptions::everything())
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(property(&fixture, "calls").as_f64(), Some(1.0));
    let actual: serde_json::Value =
        serde_wasm_bindgen::from_value(property(&fixture, "options")).unwrap();
    assert_eq!(actual, json!({ "purgeEverything": true }));
}
