import * as wasm from "./index.wasm";
import * as imports from "./index_bg.js";

// Run the worker's initialization function.
imports.start();

export default { fetch: imports.fetch, scheduled: imports.scheduled };
