import wasmModule from "./index.wasm";
import * as imports from "./index_bg.js";

const instance = new WebAssembly.Instance(wasmModule, { "./index_bg.js": imports });
export default instance.exports;