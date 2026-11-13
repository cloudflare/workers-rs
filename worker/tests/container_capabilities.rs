#![cfg(target_arch = "wasm32")]

use futures_util::FutureExt;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::wasm_bindgen_test;
use worker::{web_sys, worker_sys, Container, ContainerExecOptions, Context, Fetcher, State};

#[wasm_bindgen(module = "/tests/fixtures/container-capabilities.js")]
extern "C" {
    #[wasm_bindgen(js_name = capabilityFixture)]
    fn fixture() -> JsValue;
    #[wasm_bindgen(js_name = fixtureInput)]
    fn input(fixture: &JsValue) -> web_sys::ReadableStream;
    #[wasm_bindgen(js_name = fixtureContainer)]
    fn raw_container(fixture: &JsValue) -> worker_sys::Container;
    #[wasm_bindgen(js_name = fixtureState)]
    fn raw_state(fixture: &JsValue) -> worker_sys::DurableObjectState;
    #[wasm_bindgen(js_name = fixtureContext)]
    fn raw_context(fixture: &JsValue) -> worker_sys::Context;
    #[wasm_bindgen(js_name = fixtureProps)]
    fn props(fixture: &JsValue) -> JsValue;
    #[wasm_bindgen(js_name = pushInput)]
    fn push_input(fixture: &JsValue, bytes: &[u8]);
    #[wasm_bindgen(js_name = closeInput)]
    fn close_input(fixture: &JsValue);
    #[wasm_bindgen(js_name = failInput)]
    fn fail_input(fixture: &JsValue);
    async fn tick();
    #[wasm_bindgen(js_name = inputIdentityPreserved)]
    fn input_preserved(fixture: &JsValue) -> bool;
    fn chunks(fixture: &JsValue) -> String;
    fn completed(fixture: &JsValue) -> bool;
    #[wasm_bindgen(js_name = finishIntercept)]
    fn finish_intercept(fixture: &JsValue);
    #[wasm_bindgen(js_name = interceptTarget)]
    fn intercept_target(fixture: &JsValue) -> String;
    #[wasm_bindgen(js_name = handlerIdentityPreserved)]
    fn handler_preserved(fixture: &JsValue, handler: &JsValue) -> bool;
    #[wasm_bindgen(js_name = propsIdentityPreserved)]
    fn props_preserved(fixture: &JsValue) -> bool;
    #[wasm_bindgen(js_name = setScope)]
    fn set_scope(fixture: &JsValue, scope: &str);
    #[wasm_bindgen(js_name = pipeIdentityPreserved)]
    fn pipe_preserved(fixture: &JsValue, pipe: &web_sys::WritableStream) -> bool;
    #[wasm_bindgen(js_name = writePipe)]
    async fn write_pipe(pipe: &web_sys::WritableStream);
    #[wasm_bindgen(js_name = throwingImages)]
    fn throwing_images(fixture: &JsValue);
    #[wasm_bindgen(js_name = throwingFactory)]
    fn throwing_factory(fixture: &JsValue);
}

#[wasm_bindgen_test]
async fn exec_consumes_the_original_byte_stream_incrementally() {
    let fixture = fixture();
    let container = Container::from(raw_container(&fixture));
    let mut options = ContainerExecOptions::new();
    options.set_stdin(input(&fixture));
    let process = container
        .exec(&["synthetic-reader"], Some(options))
        .await
        .unwrap();
    assert!(input_preserved(&fixture));
    assert!(!completed(&fixture));
    assert!(process.stdin().is_none());
    push_input(&fixture, &[0, 1, 255]);
    tick().await;
    assert_eq!(chunks(&fixture), "[[0,1,255]]");
    assert!(!completed(&fixture));
    push_input(&fixture, &[2, 128]);
    tick().await;
    assert_eq!(chunks(&fixture), "[[0,1,255],[2,128]]");
    assert!(!completed(&fixture));
    close_input(&fixture);
    let output = process.output().await.unwrap();
    assert_eq!(output.stdout(), vec![0, 1, 255, 2, 128]);
    assert_eq!(output.exit_code(), 0);
    assert!(output.stderr().is_empty());
}

#[wasm_bindgen_test]
async fn exec_stream_errors_are_not_swallowed() {
    let fixture = fixture();
    let container = Container::from(raw_container(&fixture));
    let mut options = ContainerExecOptions::new();
    options.set_stdin(input(&fixture));
    let process = container
        .exec(&["synthetic-reader"], Some(options))
        .await
        .unwrap();
    fail_input(&fixture);
    assert!(process
        .output()
        .await
        .unwrap_err()
        .to_string()
        .contains("synthetic stream error"));
}

#[wasm_bindgen_test]
async fn exec_pipe_exposes_the_original_writable_capability() {
    let fixture = fixture();
    let container = Container::from(raw_container(&fixture));
    let mut options = ContainerExecOptions::new();
    options.set_stdin_pipe();
    let process = container
        .exec(&["synthetic-reader"], Some(options))
        .await
        .unwrap();
    let stdin = process.stdin().unwrap();
    assert!(pipe_preserved(&fixture, &stdin));
    write_pipe(&stdin).await;
    assert_eq!(process.output().await.unwrap().stdout(), vec![9, 0, 255]);
}

#[wasm_bindgen_test]
fn configured_image_lookup_and_errors_are_typed() {
    let fixture = fixture();
    let container = Container::from(raw_container(&fixture));
    assert_eq!(
        container.named_image("base").unwrap(),
        "registry.example.invalid/synthetic@sha256:0123456789"
    );
    assert!(container.named_image("missing").is_err());
    throwing_images(&fixture);
    assert!(container
        .named_image("base")
        .unwrap_err()
        .to_string()
        .contains("synthetic images error"));
}

async fn fetch_scope(fetcher: &Fetcher) -> String {
    let response = fetcher
        .fetch("https://synthetic.invalid/", None)
        .await
        .unwrap();
    #[cfg(feature = "http")]
    let mut response = worker::Response::try_from(response).unwrap();
    #[cfg(not(feature = "http"))]
    let mut response = response;
    response.text().await.unwrap()
}

#[wasm_bindgen_test]
async fn state_and_context_create_independently_scoped_fetchers_without_rpc_call_traps() {
    let fixture = fixture();
    let state = State::from(raw_state(&fixture));
    let context = Context::new(raw_context(&fixture));
    let first = state
        .exported_fetcher("ScopedService", props(&fixture))
        .unwrap();
    assert!(props_preserved(&fixture));
    set_scope(&fixture, "synthetic-b");
    let second = context
        .exported_fetcher("ScopedService", props(&fixture))
        .unwrap();
    assert!(props_preserved(&fixture));
    assert_eq!(fetch_scope(&first).await, "synthetic-a");
    assert_eq!(fetch_scope(&second).await, "synthetic-b");
    assert!(state.exported_fetcher("Missing", props(&fixture)).is_err());
    throwing_factory(&fixture);
    assert!(state
        .exported_fetcher("ScopedService", props(&fixture))
        .unwrap_err()
        .to_string()
        .contains("synthetic factory error"));
}

#[wasm_bindgen_test]
async fn interception_preserves_target_and_handler_and_waits_for_installation() {
    let fixture = fixture();
    let state = State::from(raw_state(&fixture));
    let handler = state
        .exported_fetcher("ScopedService", props(&fixture))
        .unwrap();
    let container = Container::from(raw_container(&fixture));
    let mut install =
        Box::pin(container.intercept_outbound_http("service.example.invalid:4000", &handler));
    assert!(install.as_mut().now_or_never().is_none());
    assert_eq!(intercept_target(&fixture), "service.example.invalid:4000");
    assert!(handler_preserved(&fixture, handler.as_ref()));
    finish_intercept(&fixture);
    install.await.unwrap();
    assert!(container
        .intercept_outbound_http("throw.invalid:4000", &handler)
        .await
        .unwrap_err()
        .to_string()
        .contains("synthetic sync intercept error"));
    assert!(container
        .intercept_outbound_http("reject.invalid:4000", &handler)
        .await
        .unwrap_err()
        .to_string()
        .contains("synthetic async intercept error"));
}

#[wasm_bindgen_test]
async fn omitted_stdin_preserves_eof_and_has_no_writable_pipe() {
    let fixture = fixture();
    let container = Container::from(raw_container(&fixture));
    let process = container.exec(&["synthetic-reader"], None).await.unwrap();
    assert!(process.stdin().is_none());
    assert!(process.output().await.unwrap().stdout().is_empty());
}

#[wasm_bindgen(module = "/tests/fixtures/container-capabilities.js")]
extern "C" {
    #[wasm_bindgen(js_name = outputIdentityPreserved)]
    fn output_preserved(
        fixture: &JsValue,
        stdout: &web_sys::ReadableStream,
        stderr: &web_sys::ReadableStream,
    ) -> bool;
    #[wasm_bindgen(js_name = streamBytes)]
    async fn stream_bytes(stream: &web_sys::ReadableStream) -> Vec<u8>;
}

#[wasm_bindgen_test]
async fn output_getters_preserve_readable_capabilities_and_optional_modes() {
    let fixture = fixture();
    let container = Container::from(raw_container(&fixture));
    let process = container.exec(&["synthetic-reader"], None).await.unwrap();
    let stdout = process.stdout().unwrap();
    let stderr = process.stderr().unwrap();
    assert!(output_preserved(&fixture, &stdout, &stderr));
    assert_eq!(stream_bytes(&stdout).await, vec![5, 0, 255]);
    assert_eq!(stream_bytes(&stderr).await, vec![6, 128]);
    let mut options = ContainerExecOptions::new();
    options.set_stdout("ignore");
    options.set_stderr("ignore");
    let ignored = container
        .exec(&["synthetic-reader"], Some(options))
        .await
        .unwrap();
    assert!(ignored.stdout().is_none());
    assert!(ignored.stderr().is_none());
    let mut options = ContainerExecOptions::new();
    options.set_stderr("combined");
    let combined = container
        .exec(&["synthetic-reader"], Some(options))
        .await
        .unwrap();
    assert!(combined.stdout().is_some());
    assert!(combined.stderr().is_none());
}

#[wasm_bindgen(module = "/tests/fixtures/container-capabilities.js")]
extern "C" {
    #[wasm_bindgen(js_name = inactivityTimeout)]
    fn inactivity_timeout(fixture: &JsValue) -> u32;
    #[wasm_bindgen(js_name = finishTimeout)]
    fn finish_timeout(fixture: &JsValue);
}

#[wasm_bindgen_test]
async fn inactivity_timeout_reapplies_to_running_containers_and_awaits_errors() {
    let fixture = fixture();
    let container = Container::from(raw_container(&fixture));
    assert!(container.running());
    let mut setting = Box::pin(container.set_inactivity_timeout(600_000));
    assert!(setting.as_mut().now_or_never().is_none());
    assert_eq!(inactivity_timeout(&fixture), 600_000);
    finish_timeout(&fixture);
    setting.await.unwrap();
    assert!(container
        .set_inactivity_timeout(0)
        .await
        .unwrap_err()
        .to_string()
        .contains("synthetic timeout must be positive"));
    assert!(container
        .set_inactivity_timeout(21_600_001)
        .await
        .unwrap_err()
        .to_string()
        .contains("synthetic timeout exceeds six hours"));
}
