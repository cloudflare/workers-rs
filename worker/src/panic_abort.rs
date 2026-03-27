use std::panic;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
thread_local! {
    // RefCell (not Cell) so we can borrow the callback without consuming it.
    // Using Cell::take() would remove the callback after the first panic, causing
    // subsequent panics in unwind-mode to silently bypass the JS hook.
    static PANIC_CALLBACK: RefCell<Option<js_sys::Function>> = RefCell::new(None);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "setPanicHook")]
pub fn set_panic_hook(callback: js_sys::Function) {
    PANIC_CALLBACK.with(|f| *f.borrow_mut() = Some(callback));
    set_once();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_panic_hook(_callback: ()) {
    // No-op on non-wasm targets
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    /// JS callback registered by the unwind shim on `globalThis` to
    /// re-initialise Durable Object instances with their stashed state/env
    /// after a wasm reinit.  Guarded with `catch` so the call is a no-op
    /// when the global is absent (e.g. abort-mode shim, or non-worker usage).
    #[wasm_bindgen(catch, js_namespace = globalThis, js_name = "__worker_reinit_dos")]
    fn worker_reinit_dos() -> Result<(), JsValue>;
}

/// Abort handler: signals that the instance should be reinitialized after a
/// hard abort (unreachable, OOM, stack overflow). Called by wasm-bindgen's JS
/// glue via the indirect function table. Only effective on `panic=unwind`
/// builds; a no-op on `panic=abort`.
#[cfg(target_arch = "wasm32")]
fn on_abort() {
    wasm_bindgen::handler::reinit();
}

/// Reinit handler: called on the NEW wasm instance after `__wbg_reset_state`
/// creates it. Re-installs the panic hook (the `std::sync::Once` is fresh on
/// a new instance) and invokes the JS-side callback that re-initialises
/// Durable Object instances with their stashed state/env.
/// Only effective on `panic=unwind` builds; a no-op on `panic=abort`.
#[cfg(target_arch = "wasm32")]
fn on_reinit() {
    set_once();
    let _ = worker_reinit_dos();
}

#[allow(deprecated)]
#[cfg(target_arch = "wasm32")]
fn hook_impl(info: &panic::PanicInfo) {
    let message = info.to_string();

    PANIC_CALLBACK.with(|f| {
        // Borrow without consuming so the callback remains registered for
        // subsequent panics (important in panic=unwind mode where the worker
        // continues after a caught panic).
        if let Some(callback) = f.borrow().as_ref() {
            use js_sys::JsString;

            if let Err(e) = callback.call1(&JsValue::UNDEFINED, &JsString::from(message.as_str())) {
                web_sys::console::error_2(&"Failed to call panic callback:".into(), &e);
            }
        } else {
            // No JS callback registered (e.g. after reinit before the shim
            // re-registers one). Fall back to logging directly.
            web_sys::console::error_1(&format!("Rust panic: {message}").into());
        }
    });
}

#[allow(deprecated)]
#[cfg(not(target_arch = "wasm32"))]
fn hook_impl(_info: &panic::PanicInfo) {
    // On non-wasm targets, we don't have contexts to abort
    // This is a no-op, but maintains the same interface
}

/// Set the WASM reinitialization panic hook the first time this is called.
/// Subsequent invocations do nothing.  On `panic=unwind` builds, also
/// registers the abort and reinit handlers via `wasm_bindgen::handler`.
#[allow(dead_code)]
#[inline]
fn set_once() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        // Register abort handler for hard-abort recovery (unreachable, OOM, etc.).
        // On panic=unwind builds this fires from __wbg_handle_catch and signals
        // reinit so the next export call creates a fresh instance.
        // On panic=abort builds these are no-ops.
        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen::handler::set_on_abort(on_abort);
            wasm_bindgen::handler::set_on_reinit(on_reinit);
        }

        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // First call the existing hook (console_error_panic_hook if set)
            default_hook(panic_info);
            hook_impl(panic_info);
        }));
    });
}
