import * as imports from "./index_bg.js";
export * from "./index_bg.js";
import wasmModule from "./index.wasm";
import { WorkerEntrypoint } from "cloudflare:workers";

// Run the worker's initialization function.
imports.start?.();

export { wasmModule };

class Entrypoint extends WorkerEntrypoint {}

Object.keys(imports).map(k => {
    Entrypoint.prototype[k] = imports[k];
})

export default Entrypoint;