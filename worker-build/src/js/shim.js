import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";
export * from "./index.js";

let panicError = null;
Error.stackTraceLimit = 100;

exports.setPanicHook(function (message) {
  panicError = new Error("Critical Rust panic: " + message);
  console.error(panicError);
});

function checkReinitialize() {
  if (panicError) {
    console.log("Reinitializing Wasm application");
    exports.__wbg_reset_state();
    panicError = null;
  }
}

class Entrypoint extends WorkerEntrypoint {
  async fetch(request) {
    checkReinitialize();
    let response = exports.fetch(request, this.env, this.ctx);
    $WAIT_UNTIL_RESPONSE;
    return await response;
  }

  async queue(batch) {
    checkReinitialize();
    return await exports.queue(batch, this.env, this.ctx);
  }

  async scheduled(event) {
    checkReinitialize();
    return await exports.scheduled(event, this.env, this.ctx);
  }
}

export default Entrypoint;
