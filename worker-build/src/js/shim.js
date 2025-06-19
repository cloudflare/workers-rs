import * as imports from "./index_bg.js";
import wasmModule from "./index.wasm";
import { WorkerEntrypoint } from "cloudflare:workers";

const instance = new WebAssembly.Instance(wasmModule, {
	"./index_bg.js": imports,
});

imports.__wbg_set_wasm(instance.exports);

// Run the worker's initialization function.
instance.exports.__wbindgen_start?.();

$DURABLE_OBJECTS_INJECTION_POINT

class Entrypoint extends WorkerEntrypoint {
	async fetch(request) {
		let response = imports.fetch(request, this.env, this.ctx);
		$WAIT_UNTIL_RESPONSE;
		return await response;
	}

	async queue(batch) {
		return await imports.queue(batch, this.env, this.ctx);
	}

	async scheduled(event) {
		return await imports.scheduled(event, this.env, this.ctx);
	}
}

const EXCLUDE_EXPORT = [
	...successfullyWrappedDONames,
	"IntoUnderlyingByteSource",
	"IntoUnderlyingSink",
	"IntoUnderlyingSource",
	"MinifyConfig",
	"PolishConfig",
	"R2Range",
	"RequestRedirect",
	"fetch",
	"queue",
	"scheduled",
	"getMemory",
];

Object.keys(imports).forEach((k) => {
	if (!(EXCLUDE_EXPORT.includes(k) | k.startsWith("__"))) {
		Entrypoint.prototype[k] = imports[k];
	}
});

export default Entrypoint;
export { wasmModule };
