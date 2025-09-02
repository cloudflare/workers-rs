use js_sys::{Object, Promise};
use wasm_bindgen::prelude::*;

use crate::Fetcher;

/// ```ts
/// interface Container {
///   get running(): boolean;
///   start(options?: ContainerStartupOptions): void;
///   monitor(): Promise<void>;
///   destroy(error?: any): Promise<void>;
///   signal(signo: number): void;
///   getTcpPort(port: number): Fetcher;
/// }
///
/// interface ContainerStartupOptions {
///   entrypoint?: string[];
///   enableInternet: boolean;
///   env?: Record<string, string>;
/// }
/// ```

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Container;

    #[wasm_bindgen(method, getter)]
    pub fn running(this: &Container) -> bool;

    #[wasm_bindgen(method)]
    pub fn start(this: &Container, options: Option<&ContainerStartupOptions>);

    #[wasm_bindgen(method)]
    pub fn monitor(this: &Container) -> Promise;

    #[wasm_bindgen(method)]
    pub fn destroy(this: &Container, error: &JsValue) -> Promise;

    #[wasm_bindgen(method)]
    pub fn signal(this: &Container, signo: i32);

    #[wasm_bindgen(method, catch, js_name=getTcpPort)]
    pub fn get_tcp_port(this: &Container, port: u16) -> Result<Fetcher, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ContainerStartupOptions;

    #[wasm_bindgen(method, getter)]
    pub fn entrypoint(this: &ContainerStartupOptions) -> Option<Vec<String>>;

    #[wasm_bindgen(method, getter, js_name=enableInternet)]
    pub fn enable_internet(this: &ContainerStartupOptions) -> bool;

    #[wasm_bindgen(method, catch, getter)]
    pub fn env(this: &ContainerStartupOptions) -> Result<Object, JsValue>;
}
