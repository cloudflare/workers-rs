import wasmModule from "./index.wasm";
import * as imports from "./index_bg.js";
/* #IMPORTS_JS_SNIPPETS# */

const instance = new WebAssembly.Instance(wasmModule, { "./index_bg.js": imports/* #WASM_JS_SNIPPETS# */ });
export default instance.exports;
