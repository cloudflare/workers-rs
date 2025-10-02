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

let instanceId = 0;
function checkReinitialize() {
  if (panicError) {
    console.log("Reinitializing Wasm application");
    exports.__wbg_reset_state();
    panicError = null;
    registerPanicHook();
    instanceId++;
  }
}

export default class Entrypoint extends WorkerEntrypoint {
$HANDLERS
}

const classProxyHooks = {
  construct(ctor, args, newTarget) {
    const instance = {
      instance: Reflect.construct(ctor, args, newTarget),
      instanceId,
      ctor,
      args,
      newTarget
    };
    return new Proxy(instance, {
      get(target, prop, receiver) {
        if (target.instanceId !== instanceId) {
          target.instance = Reflect.construct(target.ctor, target.args, target.newTarget);
          target.instanceId = instanceId;
        }
        return Reflect.get(target.instance, prop, receiver);
      }
    });
  }
};
