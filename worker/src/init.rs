use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export const state = globalThis.__worker_init_state = { criticalError: false, instanceId: 0 };
")]
extern "C" {
    type InitState;

    #[wasm_bindgen(thread_local_v2, js_name = state)]
    static INIT_STATE: InitState;

    #[cfg(not(panic = "unwind"))]
    #[wasm_bindgen(method, setter, js_name = criticalError)]
    fn set_critical_error(this: &InitState, val: bool);

    #[wasm_bindgen(method, getter, js_name = instanceId)]
    fn instance_id(this: &InitState) -> u32;

    #[wasm_bindgen(method, setter, js_name = instanceId)]
    fn set_instance_id(this: &InitState, val: u32);
}

// On abort -> reinit
fn on_abort() {
    wasm_bindgen::handler::schedule_reinit();
}

// When building with panic=unwind and reinitialization support,
// this hook will be called for any critical error or abort.
// Since wasm-bindgen manages the reinitialization for us in this
// case, we only need to bump the instance ID.
fn on_reinit() {
    INIT_STATE.with(|s| {
        let id = s.instance_id();
        s.set_instance_id(id + 1);
    });
}

#[wasm_bindgen(start)]
fn init() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        default_hook(info);
        let message = info.to_string();
        let panic_err = js_sys::Error::new(&format!("Rust panic: {message}"));
        web_sys::console::error_2(&"Critical".into(), &panic_err.into());
        #[cfg(not(panic = "unwind"))]
        {
            INIT_STATE.with(|s| s.set_critical_error(true));
        }
    }));

    wasm_bindgen::handler::set_on_abort(on_abort);
    wasm_bindgen::handler::set_on_reinit(on_reinit);
}

#[wasm_bindgen(js_name = "__worker_init_state")]
pub fn worker_init_state() -> JsValue {
    INIT_STATE.with(|s| s.into())
}
