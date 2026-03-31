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

function panicHook(message) {
  console.error("Rust panic:", message);
}

function registerPanicHook() {
  if (exports.setPanicHook) {
    exports.setPanicHook(panicHook);
  }
}

registerPanicHook();

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

$DO_CLASSES

export default Entrypoint;
