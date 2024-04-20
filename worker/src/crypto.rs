use wasm_bindgen::JsCast;

pub fn crypto() -> web_sys::Crypto {
    let global: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();
    global.crypto().expect("failed to acquire Crypto")
}
