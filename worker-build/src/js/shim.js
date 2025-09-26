import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";
export * from "./index.js";

Error.stackTraceLimit = 100;

let panicError = null;
function registerPanicHook() {
  if (exports.setPanicHook)
    exports.setPanicHook(function (message) {
      panicError = new Error("Critical Rust panic: " + message);
      console.error(panicError);
    });
}

registerPanicHook();

let reinitId = 0;
function checkReinitialize() {
  if (panicError) {
    console.log("Reinitializing Wasm application");
    exports.__wbg_reset_state();
    panicError = null;
    registerPanicHook();
    reinitId++;
  }
}

export default class Entrypoint extends WorkerEntrypoint {
$HANDLERS
}

const instances = new Map();
const classProxyHooks = {
  construct(ctor, args, newTarget) {
    instances.get(ctor)?.free();
    const instance = Reflect.construct(ctor, args, newTarget);
    instances.set(ctor, instance);
    const target = {
      instance,
      reinitId,
      ctor,
      args,
      newTarget
    };
    return new Proxy(target, {
      get(target, prop, receiver) {
        if (target.reinitId !== reinitId) {
          instances.set(target.ctor, target.instance = Reflect.construct(target.ctor, target.args, target.newTarget));
          target.reinitId = reinitId;
        }
        return Reflect.get(target.instance, prop, receiver);
      }
    });
  }
};
