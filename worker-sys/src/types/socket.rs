use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "cloudflare:sockets")]
extern "C" {
    #[wasm_bindgen]
    pub fn connect(address: JsValue, options: JsValue) -> Socket;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Socket;

    #[wasm_bindgen(method)]
    pub fn close(this: &Socket) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn closed(this: &Socket) -> js_sys::Promise;

    #[wasm_bindgen(method, js_name=startTls)]
    pub fn start_tls(this: &Socket) -> Socket;
}

impl Socket {
    pub fn readable(&self) -> ReadableStream {
        js_sys::Reflect::get(self, &JsValue::from("readable"))
            .unwrap()
            .into()
    }

    pub fn writable(&self) -> WritableStream {
        js_sys::Reflect::get(self, &JsValue::from("writable"))
            .unwrap()
            .into()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ReadableStream;

    #[wasm_bindgen(method, js_name=getReader)]
    pub fn get_reader(this: &ReadableStream) -> ReadableStreamDefaultReader;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ReadableStreamDefaultReader;

    #[wasm_bindgen(method)]
    pub fn read(this: &ReadableStreamDefaultReader) -> js_sys::Promise;

    #[wasm_bindgen(method, js_name=releaseLock)]
    pub fn release_lock(this: &ReadableStreamDefaultReader);
}

impl ReadableStreamDefaultReader {
    pub fn closed(&self) -> js_sys::Promise {
        js_sys::Reflect::get(self, &JsValue::from("closed"))
            .unwrap()
            .into()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WritableStream;

    #[wasm_bindgen(method, js_name=getWriter)]
    pub fn get_writer(this: &WritableStream) -> WritableStreamDefaultWriter;
}

impl WritableStream {
    pub fn locked(&self) -> bool {
        js_sys::Reflect::get(self, &JsValue::from("locked"))
            .unwrap()
            .as_bool()
            .unwrap()
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WritableStreamDefaultWriter;

    #[wasm_bindgen(method)]
    pub fn write(this: &WritableStreamDefaultWriter, chunk: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    pub fn close(this: &WritableStreamDefaultWriter) -> js_sys::Promise;

    #[wasm_bindgen(method, js_name=releaseLock)]
    pub fn release_lock(this: &WritableStreamDefaultWriter);

    #[wasm_bindgen(method)]
    pub fn abort(this: &WritableStreamDefaultWriter, reason: String) -> js_sys::Promise;
}

impl WritableStreamDefaultWriter {
    pub fn closed(&self) -> js_sys::Promise {
        js_sys::Reflect::get(self, &JsValue::from("closed"))
            .unwrap()
            .into()
    }

    pub fn desired_size(&self) -> usize {
        js_sys::Reflect::get(self, &JsValue::from("desiredSize"))
            .unwrap()
            .as_f64()
            .unwrap() as usize
    }
}
