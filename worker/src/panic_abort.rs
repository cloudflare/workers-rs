use std::panic;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use std::cell::Cell;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static PANIC_CALLBACK: Cell<Option<js_sys::Function>> = Cell::new(None);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "setPanicHook")]
pub fn set_panic_hook(callback: js_sys::Function) {
    PANIC_CALLBACK.with(|f| f.set(Some(callback)));
    set_once();
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_panic_hook(_callback: ()) {
    // No-op on non-wasm targets
}

#[allow(deprecated)]
#[cfg(target_arch = "wasm32")]
fn hook_impl(info: &panic::PanicInfo) {
    let message = info.to_string();

    PANIC_CALLBACK.with(|f| {
        if let Some(callback) = f.take() {
            use js_sys::JsString;

            if let Err(e) = callback.call1(&JsValue::UNDEFINED, &JsString::from(message)) {
                web_sys::console::error_2(&"Failed to call panic callback:".into(), &e);
            }
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
/// Subsequent invocations do nothing.
#[allow(dead_code)]
#[inline]
fn set_once() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        let default_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            // First call the existing hook (console_error_panic_hook if set)
            default_hook(panic_info);
            hook_impl(panic_info);
        }));
    });
}
