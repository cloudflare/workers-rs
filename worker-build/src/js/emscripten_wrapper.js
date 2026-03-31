import createModule from './output.js';
import wasmModule from './output.wasm';

let modulePromise = null;

function getModule() {
  if (!modulePromise) {
    modulePromise = new Promise((resolve, reject) => {
      createModule({
        instantiateWasm(imports, successCallback) {
          WebAssembly.instantiate(wasmModule, imports).then(
            instance => { successCallback(instance, wasmModule); },
            reject
          );
          return {};
        },
      }).then(resolve, reject);
    });
  }
  return modulePromise;
}

$CLASSES
export default {
$HANDLERS};
