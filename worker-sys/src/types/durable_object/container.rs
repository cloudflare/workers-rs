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
///   interceptOutboundHttp(target: string, worker: Fetcher): Promise<void>;
///   interceptAllOutboundHttp(worker: Fetcher): Promise<void>;
///   interceptOutboundHttps(target: string, worker: Fetcher): Promise<void>;
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
    #[wasm_bindgen(extends=Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Container;

    #[wasm_bindgen(method, getter)]
    pub fn running(this: &Container) -> bool;

    #[wasm_bindgen(method, catch)]
    pub fn start(this: &Container, options: &JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(method)]
    pub fn monitor(this: &Container) -> Promise;

    #[wasm_bindgen(method)]
    pub fn destroy(this: &Container, error: Option<&str>) -> Promise;

    #[wasm_bindgen(method, catch)]
    pub fn signal(this: &Container, signo: i32) -> Result<(), JsValue>;

    #[wasm_bindgen(method, catch, js_name=getTcpPort)]
    pub fn get_tcp_port(this: &Container, port: u16) -> Result<Fetcher, JsValue>;

    #[wasm_bindgen(method, catch, js_name=interceptOutboundHttp)]
    pub fn intercept_outbound_http(
        this: &Container,
        target: &str,
        worker: &Fetcher,
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=interceptAllOutboundHttp)]
    pub fn intercept_all_outbound_http(
        this: &Container,
        worker: &Fetcher,
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=interceptOutboundHttps)]
    pub fn intercept_outbound_https(
        this: &Container,
        target: &str,
        worker: &Fetcher,
    ) -> Result<Promise, JsValue>;
}
