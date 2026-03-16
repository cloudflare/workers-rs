import createModule from './output.js';
import wasmModule from './output.wasm';

let modulePromise = null;

function getModule() {
  if (!modulePromise) {
    modulePromise = createModule({
      instantiateWasm(imports, successCallback) {
        WebAssembly.instantiate(wasmModule, imports).then(instance => {
          successCallback(instance, wasmModule);
        });
        return {};
      },
    });
  }
  return modulePromise;
}

export default {
$HANDLERS};
