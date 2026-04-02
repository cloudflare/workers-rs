//! Wasm instance generation tracking for Durable Object stale-instance
//! detection.
//!
//! After a Wasm reset (`__wbg_reset_state`), class instances from the previous
//! Wasm instance become stale.  The JS shim needs a way to detect this so it
//! can re-construct Durable Object instances transparently.
//!
//! This module maintains a generation counter in an object stored on
//! `globalThis` that persists across Wasm resets.  A `set_on_reinit` hook bumps
//! the counter each time a fresh instance is created.  The shim grabs a
//! reference to the shared object once at module load time, then compares its
//! `.id` property (a pure JS property read — no Wasm hop) at each method call.

use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// JS-side persistent state object.  Stored on `globalThis` so it survives
// `__wbg_reset_state` without needing an `inline_js` snippet (which causes
// esbuild duplicate-key warnings when wasm-bindgen emits the module map).
// ---------------------------------------------------------------------------

const STATE_KEY: &str = "__worker_reinit_state";

fn get_or_create_state() -> JsValue {
    let global = js_sys::global();
    let key = JsValue::from_str(STATE_KEY);
    let existing = js_sys::Reflect::get(&global, &key).unwrap_or(JsValue::UNDEFINED);
    if !existing.is_undefined() {
        return existing;
    }
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &JsValue::from_str("id"), &JsValue::from(0_u32)).expect("set id");
    js_sys::Reflect::set(&global, &key, &obj).expect("set state on globalThis");
    obj.into()
}

fn bump_instance_id() {
    let state = get_or_create_state();
    let current = js_sys::Reflect::get(&state, &JsValue::from_str("id"))
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0) as u32;
    js_sys::Reflect::set(
        &state,
        &JsValue::from_str("id"),
        &JsValue::from(current + 1),
    )
    .expect("bump id");
}

// ---------------------------------------------------------------------------
// Reinit hook — called by wasm-bindgen after each fresh instance is created.
// ---------------------------------------------------------------------------

fn on_reinit() {
    web_sys::console::log_1(&"[workers-rs] reinit".into());
    bump_instance_id();
}

/// Register the reinit hook that bumps the JS-side instance counter.
///
/// Uses `Once` internally so repeated calls are harmless.  After a Wasm reset
/// the `Once` is fresh (new instance), so the hook is re-registered
/// automatically when `__wbindgen_start` runs the start function again.
fn ensure_initialized() {
    use std::sync::Once;
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        web_sys::console::log_1(&"[workers-rs] registering reinit hook".into());
        wasm_bindgen::handler::set_on_reinit(on_reinit);
    });
}

/// Install a panic hook that logs to `console.error` with a full message.
/// For `panic=unwind` builds the simplified shim does not call `setPanicHook`,
/// so this is the only source of readable panic output.  Uses `Once` so it is
/// safe to call on every fresh instance after reinit.
fn set_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        #[allow(deprecated)]
        std::panic::set_hook(Box::new(|info| {
            web_sys::console::error_1(&info.to_string().into());
        }));
    });
}

/// Initialize panic hook and reinit tracking.  Called from the `#[event]`
/// macro-generated code — either from the `#[event(start)]` wrapper or lazily
/// at the top of the first handler invocation.  Safe to call multiple times;
/// each component uses `Once` internally.
pub fn init() {
    set_panic_hook();
    ensure_initialized();
}

// ---------------------------------------------------------------------------
// Export for the JS shim.
// ---------------------------------------------------------------------------

/// Returns the shared state object `{ id: number }` from `globalThis`.  The
/// shim calls this once at module load time and then reads `state.id` directly
/// on every DO method call — a plain JS property access with no Wasm boundary
/// crossing.
#[wasm_bindgen(js_name = "__worker_reinit_state")]
pub fn reinit_state() -> JsValue {
    get_or_create_state()
}
