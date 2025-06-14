import * as imports from "./index_bg.js";
export * from "./index_bg.js";
import wasmModule from "./index.wasm";
import { WorkerEntrypoint } from "cloudflare:workers";

const instance = new WebAssembly.Instance(wasmModule, {
	"./index_bg.js": imports,
});

imports.__wbg_set_wasm(instance.exports);

// Run the worker's initialization function.
instance.exports.__wbindgen_start?.();

export { wasmModule };

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
	"Rpc"
];

Object.keys(imports).forEach((key) => {
	const fn = imports[key];
	if (typeof fn === "function" && !EXCLUDE_EXPORT.includes(key) && !key.startsWith("__")) {
		// Otherwise, assign the function as-is.
		Entrypoint.prototype[key] = fn;
	}
});


// Helper to lazily create the RPC instance
Entrypoint.prototype._getRpc = function (Ctor) {
  if (!this._rpcInstanceMap) this._rpcInstanceMap = new Map();
  if (!this._rpcInstanceMap.has(Ctor)) {
    this._rpcInstanceMap.set(Ctor, new Ctor(this.env));
  }
  return this._rpcInstanceMap.get(Ctor);
};

const EXCLUDE_RPC_EXPORT = ["constructor", "new", "free"];

//Register RPC entrypoint methods into Endpoint
Object.entries(imports).forEach(([exportName, exportValue]) => {
  if (typeof exportValue === "function" && exportValue.prototype?.__is_rpc__) {
    const Ctor = exportValue;

    const methodNames = Object.getOwnPropertyNames(Ctor.prototype)
      .filter(name => !EXCLUDE_RPC_EXPORT.includes(name) && typeof exportValue.prototype[name] === "function");

    for (const methodName of methodNames) {
      if (!Entrypoint.prototype.hasOwnProperty(methodName)) {
        Entrypoint.prototype[methodName] = function (...args) {
          const rpc = this._getRpc(Ctor);
          return rpc[methodName](...args);
        };
      }
    }
  }
});


export default Entrypoint;
