use wasm_bindgen::prelude::*;
use js_sys::{JsString, Object as JsObject};

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(extends=JsObject)]
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub type TailItem;

	#[wasm_bindgen(method, getter)]
	pub fn event(this: &TailItem) -> JsObject;

	#[wasm_bindgen(method, getter, js_name=eventTimestamp)]
	pub fn event_timestamp(this: &TailItem) -> Option<i64>;

	#[wasm_bindgen(method, getter)]
	pub fn logs(this: &TailItem) -> Vec<TailLog>;

	#[wasm_bindgen(method, getter)]
	pub fn exceptions(this: &TailItem) -> Vec<TailException>;

	#[wasm_bindgen(method, getter, js_name=scriptName)]
	pub fn script_name(this: &TailItem) -> Option<String>;

	#[wasm_bindgen(method, getter, js_name=dispatchNamespace)]
	pub fn dispatch_namespace(this: &TailItem) -> Option<String>;

	#[wasm_bindgen(method, getter, js_name=scriptTags)]
	pub fn script_tags(this: &TailItem) -> Option<Vec<JsString>>;

	#[wasm_bindgen(method, getter)]
	pub fn outcome(this: &TailItem) -> String;
}

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(extends=JsObject)]
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub type TailException;

	#[wasm_bindgen(method, getter)]
	fn timestamp(this: &TailException) -> i64;

	#[wasm_bindgen(method, getter)]
	fn message(this: &TailException) -> String;

	#[wasm_bindgen(method, getter)]
	fn name(this: &TailException) -> String;
}

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(extends=JsObject)]
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub type TailLog;

	#[wasm_bindgen(method, getter)]
	pub fn timestamp(this: &TailLog) -> i64;

	#[wasm_bindgen(method, getter)]
	pub fn level(this: &TailLog) -> String;

	#[wasm_bindgen(method, getter)]
	pub fn message(this: &TailLog) -> Vec<JsValue>;
}