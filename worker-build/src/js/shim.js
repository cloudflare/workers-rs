import * as imports from "./index_bg.js";
export * from "./index_bg.js";
import wasmModule from "./index.wasm";

// Run the worker's initialization function.
imports.start?.();

export { wasmModule };
export default { fetch: imports.fetch, scheduled: imports.scheduled, queue: imports.queue };
