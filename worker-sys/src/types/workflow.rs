use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WorkflowStep;

    #[wasm_bindgen(method, catch, js_name = "do")]
    pub fn do_(
        this: &WorkflowStep,
        name: &str,
        callback: &js_sys::Function,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "do")]
    pub fn do_with_config(
        this: &WorkflowStep,
        name: &str,
        config: JsValue,
        callback: &js_sys::Function,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn sleep(
        this: &WorkflowStep,
        name: &str,
        duration: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = sleepUntil)]
    pub fn sleep_until(
        this: &WorkflowStep,
        name: &str,
        timestamp: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = waitForEvent)]
    pub fn wait_for_event(
        this: &WorkflowStep,
        name: &str,
        options: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;

    /// Workflow binding type - may be a Workflow object, WorkflowImpl, or Fetcher (RPC stub).
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WorkflowBinding;

    #[wasm_bindgen(method, catch)]
    pub fn get(this: &WorkflowBinding, id: &str) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn create(this: &WorkflowBinding, options: JsValue) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = createBatch)]
    pub fn create_batch(
        this: &WorkflowBinding,
        batch: &js_sys::Array,
    ) -> Result<js_sys::Promise, JsValue>;

    /// Workflow instance handle - may be an RPC stub in Miniflare.
    #[wasm_bindgen(extends=js_sys::Object)]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type WorkflowInstanceSys;

    #[wasm_bindgen(method, catch)]
    pub fn pause(this: &WorkflowInstanceSys) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn resume(this: &WorkflowInstanceSys) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn terminate(this: &WorkflowInstanceSys) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn restart(this: &WorkflowInstanceSys) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch)]
    pub fn status(this: &WorkflowInstanceSys) -> Result<js_sys::Promise, JsValue>;

    #[wasm_bindgen(method, catch, js_name = sendEvent)]
    pub fn send_event(
        this: &WorkflowInstanceSys,
        event: JsValue,
    ) -> Result<js_sys::Promise, JsValue>;
}
