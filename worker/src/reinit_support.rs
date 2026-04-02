//! Wasm instance generation tracking for Durable Object stale-instance
//! detection.
//!
//! After a Wasm reset (`__wbg_reset_state`), class instances from the previous
//! Wasm instance become stale.  The JS shim needs a way to detect this so it
//! can re-construct Durable Object instances transparently.
//!
//! This module maintains a generation counter in a shared JS object (via
//! `inline_js`) that persists across Wasm resets.  A `set_on_reinit` hook bumps
//! the counter each time a fresh instance is created.  The shim grabs a
//! reference to the shared object once at module load time, then compares its
//! `.id` property (a pure JS property read — no Wasm hop) at each method call.

use wasm_bindgen::prelude::*;

// ---------------------------------------------------------------------------
// JS-side persistent state object.  The inline snippet lives outside the Wasm
// instance so its state survives `__wbg_reset_state`.  The shim holds a
// reference to the same object and reads `.id` directly — zero Wasm overhead
// on the hot path.
// ---------------------------------------------------------------------------

#[wasm_bindgen(inline_js = "
const state = { id: 0 };
export function bumpInstanceId() { state.id++; }
export function getState() { return state; }
")]
extern "C" {
    #[wasm_bindgen(js_name = "bumpInstanceId")]
    fn bump_instance_id();

    #[wasm_bindgen(js_name = "getState")]
    fn get_state() -> JsValue;
}

// ---------------------------------------------------------------------------
// Reinit hook — called by wasm-bindgen after each fresh instance is created.
// ---------------------------------------------------------------------------

fn on_reinit() {
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

/// Returns the shared state object `{ id: number }`.  The shim calls this once
/// at module load time and then reads `state.id` directly on every DO method
/// call — a plain JS property access with no Wasm boundary crossing.
#[wasm_bindgen(js_name = "__worker_reinit_state")]
pub fn reinit_state() -> JsValue {
    get_state()
}
