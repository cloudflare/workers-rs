//! `console_log!` and friends: `format!`-style macros over the generated `console`
//! global. The formatted text is passed as a single argument, so the console sees one
//! string rather than a Rust format spec.

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {
        $crate::console.with(|c| c.log(&[$crate::wasm_bindgen::JsValue::from(::std::format!($($t)*))]))
    };
}

#[macro_export]
macro_rules! console_debug {
    ($($t:tt)*) => {
        $crate::console.with(|c| c.debug(&[$crate::wasm_bindgen::JsValue::from(::std::format!($($t)*))]))
    };
}

#[macro_export]
macro_rules! console_warn {
    ($($t:tt)*) => {
        $crate::console.with(|c| c.warn(&[$crate::wasm_bindgen::JsValue::from(::std::format!($($t)*))]))
    };
}

#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => {
        $crate::console.with(|c| c.error(&[$crate::wasm_bindgen::JsValue::from(::std::format!($($t)*))]))
    };
}
