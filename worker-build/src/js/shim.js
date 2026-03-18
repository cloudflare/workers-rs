import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";

Error.stackTraceLimit = 100;

let criticalError = false;
function registerPanicHook() {
  if (exports.setPanicHook)
    exports.setPanicHook(function (message) {
      const panicError = new Error("Rust panic: " + message);
      console.error('Critical', panicError);
      criticalError = true;
    });
}

registerPanicHook();

let instanceId = 0;
function checkReinitialize() {
  if (criticalError) {
    console.log("Reinitializing Wasm application");
    exports.__wbg_reset_state();
    criticalError = false;
    registerPanicHook();
    instanceId++;
  }
}

addEventListener('error', (e) => {
  handleMaybeCritical(e.error);
});

function handleMaybeCritical(e) {
  if (e instanceof WebAssembly.RuntimeError) {
    console.error('Critical', e);
    criticalError = true;
  }
}

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

$DO_CLASSES

export default Entrypoint;
