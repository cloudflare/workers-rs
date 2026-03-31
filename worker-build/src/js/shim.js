import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";

Error.stackTraceLimit = 100;

// Polyfill console.createTask for runtimes that don't support it (e.g. workerd).
// wasm-bindgen-futures calls this under debug_assertions for async task tracking.
if (typeof console.createTask !== "function") {
  console.createTask = (_name) => ({
    run: (fn) => fn(),
  });
}

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
    // The try/catch guards against panic=abort builds where __reinit_handler
    // is not present as a Wasm global — the fresh instance is still created
    // successfully before the handler invocation that may throw.
    try { exports.__wbg_reset_state(); } catch(_) {}
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
