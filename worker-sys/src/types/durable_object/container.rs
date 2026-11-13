use js_sys::{ArrayBuffer, Object, Promise};
use wasm_bindgen::prelude::*;

use crate::Fetcher;

/// ```ts
/// interface Container {
///   get running(): boolean;
///   get images(): Record<string, string>;
///   start(options?: ContainerStartupOptions): void;
///   monitor(): Promise<void>;
///   destroy(error?: any): Promise<void>;
///   signal(signo: number): void;
///   getTcpPort(port: number): Fetcher;
///   exec(cmd: string[], options?: ContainerExecOptions): Promise<ExecProcess>;
///   interceptOutboundHttp(addr: string, binding: Fetcher): Promise<void>;
/// }
///
/// interface ContainerStartupOptions {
///   entrypoint?: string[];
///   enableInternet: boolean;
///   env?: Record<string, string>;
///   // `durable_object` scheduling policy only.
///   image?: string;
///   instance?: string | { vcpu: number; memoryMib: number; diskMb: number };
///   containerSnapshot?: ContainerSnapshotRestoreParams;
///   labels?: Record<string, string>;
/// }
///
/// interface ExecProcess {
///   readonly stdin: WritableStream | undefined;
///   readonly pid: number;
///   readonly exitCode: Promise<number>;
///   output(): Promise<ExecOutput>;
/// }
///
/// interface ExecOutput {
///   readonly exitCode: number;
///   readonly stdout: ArrayBuffer;
///   readonly stderr: ArrayBuffer;
/// }
/// ```

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type Container;

    #[wasm_bindgen(method, getter)]
    pub fn running(this: &Container) -> bool;

    #[wasm_bindgen(method, catch, getter)]
    pub fn images(this: &Container) -> Result<Object, JsValue>;

    #[wasm_bindgen(method, catch, js_name=interceptOutboundHttp)]
    pub fn intercept_outbound_http(
        this: &Container,
        addr: &str,
        binding: &Fetcher,
    ) -> Result<Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name=setInactivityTimeout)]
    pub fn set_inactivity_timeout(this: &Container, duration_ms: u32) -> Result<Promise, JsValue>;

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

    /// Starts another process inside an already-running container. `cmd` is the
    /// executable followed by its arguments; `options` is a `ContainerExecOptions`
    /// object (or `undefined`).
    #[wasm_bindgen(method, catch)]
    pub fn exec(this: &Container, cmd: Vec<JsValue>, options: &JsValue)
        -> Result<Promise, JsValue>;

    #[wasm_bindgen(extends=Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ExecProcess;

    #[wasm_bindgen(method, getter)]
    pub fn stdin(this: &ExecProcess) -> Option<web_sys::WritableStream>;

    #[wasm_bindgen(method, getter)]
    pub fn stdout(this: &ExecProcess) -> Option<web_sys::ReadableStream>;

    #[wasm_bindgen(method, getter)]
    pub fn stderr(this: &ExecProcess) -> Option<web_sys::ReadableStream>;

    #[wasm_bindgen(method, getter, js_name=pid)]
    pub fn pid(this: &ExecProcess) -> u32;

    /// Resolves with the process's exit code when it exits. Nonzero codes
    /// resolve normally instead of rejecting.
    #[wasm_bindgen(method, getter, js_name=exitCode)]
    pub fn exit_code(this: &ExecProcess) -> Promise;

    /// Reads buffered output once. Throws (via the rejected `Promise`) when
    /// called more than once or after a readable stream has been consumed.
    #[wasm_bindgen(method, catch)]
    pub fn output(this: &ExecProcess) -> Result<Promise, JsValue>;

    /// Queues a signal for the process; default is `SIGTERM` (15).
    #[wasm_bindgen(method, catch)]
    pub fn kill(this: &ExecProcess, signal: Option<i32>) -> Result<(), JsValue>;

    #[wasm_bindgen(extends=Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ExecOutput;

    #[wasm_bindgen(method, getter, js_name=exitCode)]
    pub fn exit_code(this: &ExecOutput) -> u32;

    #[wasm_bindgen(method, getter, js_name=stdout)]
    pub fn stdout(this: &ExecOutput) -> ArrayBuffer;

    #[wasm_bindgen(method, getter, js_name=stderr)]
    pub fn stderr(this: &ExecOutput) -> ArrayBuffer;
}
