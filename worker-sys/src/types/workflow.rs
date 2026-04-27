use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WorkflowStep;

    #[wasm_bindgen(method, catch, js_name = "do")]
    pub async fn do_(
        this: &WorkflowStep,
        name: &str,
        callback: &js_sys::Function,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "do")]
    pub async fn do_with_config(
        this: &WorkflowStep,
        name: &str,
        config: JsValue,
        callback: &js_sys::Function,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub async fn sleep(
        this: &WorkflowStep,
        name: &str,
        duration: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = sleepUntil)]
    pub async fn sleep_until(
        this: &WorkflowStep,
        name: &str,
        timestamp: JsValue,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = waitForEvent)]
    pub async fn wait_for_event(
        this: &WorkflowStep,
        name: &str,
        options: JsValue,
    ) -> Result<JsValue, JsValue>;

    /// Workflow binding type - may be a Workflow object, WorkflowImpl, or Fetcher (RPC stub).
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WorkflowBinding;

    #[wasm_bindgen(method, catch)]
    pub async fn get(this: &WorkflowBinding, id: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub async fn create(this: &WorkflowBinding, options: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = createBatch)]
    pub async fn create_batch(
        this: &WorkflowBinding,
        batch: &js_sys::Array,
    ) -> Result<JsValue, JsValue>;

    /// Workflow instance handle - may be an RPC stub in Miniflare.
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WorkflowInstanceSys;

    #[wasm_bindgen(method, catch)]
    pub async fn pause(this: &WorkflowInstanceSys) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub async fn resume(this: &WorkflowInstanceSys) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub async fn terminate(this: &WorkflowInstanceSys) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub async fn restart(this: &WorkflowInstanceSys) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub async fn status(this: &WorkflowInstanceSys) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = sendEvent)]
    pub async fn send_event(this: &WorkflowInstanceSys, event: JsValue)
        -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(module = "cloudflare:workflows")]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Error)]
    #[derive(Debug, Clone)]
    pub type NonRetryableErrorSys;

    #[wasm_bindgen(constructor, js_class = "NonRetryableError")]
    pub fn new(message: &str) -> NonRetryableErrorSys;
}
