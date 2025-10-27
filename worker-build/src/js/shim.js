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

addEventListener('unhandledRejection', (e) => {
  handleMaybeCritical(e.error);
});

addEventListener('error', (e) => {
  handleMaybeCritical(e.error);
});

function handleMaybeCritical(e) {
  if (e instanceof WebAssembly.RuntimeError) {
    console.error('Critical', e);
    criticalError = true;
  }
}

class Entrypoint extends WorkerEntrypoint {
$HANDLERS
}

const instanceProxyHooks = {
  set: (target, prop, value, receiver) => Reflect.set(target.instance, prop, value, receiver),
  has: (target, prop) => Reflect.has(target.instance, prop),
  deleteProperty: (target, prop) => Reflect.deleteProperty(target.instance, prop),
  apply: (target, thisArg, args) => Reflect.apply(target.instance, thisArg, args),
  construct: (target, args, newTarget) => Reflect.construct(target.instance, args, newTarget),
  getPrototypeOf: (target) => Reflect.getPrototypeOf(target.instance),
  setPrototypeOf: (target, proto) => Reflect.setPrototypeOf(target.instance, proto),
  isExtensible: (target) => Reflect.isExtensible(target.instance),
  preventExtensions: (target) => Reflect.preventExtensions(target.instance),
  getOwnPropertyDescriptor: (target, prop) => Reflect.getOwnPropertyDescriptor(target.instance, prop),
  defineProperty: (target, prop, descriptor) => Reflect.defineProperty(target.instance, prop, descriptor),
  ownKeys: (target) => Reflect.ownKeys(target.instance),
};

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
      ...instanceProxyHooks,
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

export default new Proxy(Entrypoint, classProxyHooks);
