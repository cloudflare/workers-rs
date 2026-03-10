import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";

Error.stackTraceLimit = 100;

// Best-effort panic logging. Lost after wasm reinit since
// wasm-bindgen calls __wbg_reset_state internally and the
// shim has no hook to re-register.
if (exports.setPanicHook) {
  exports.setPanicHook(function (message) {
    console.error("Rust panic:", message);
  });
}

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

export default Entrypoint;
