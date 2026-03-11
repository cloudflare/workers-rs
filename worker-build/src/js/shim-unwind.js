import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";

Error.stackTraceLimit = 100;

let instanceId = 0;

function panicHook(message) {
  console.error("Rust panic:", message);
}

function registerPanicHook() {
  if (exports.setPanicHook) {
    exports.setPanicHook(panicHook);
  }
}

registerPanicHook();

// After wasm-bindgen auto-reinits via __wbg_reset_state, this hook fires
// so we can re-register the panic hook and bump the instance generation
// counter (used by the class Proxy to reconstruct stale DO instances).
if (exports.__wbg_set_reinit_hook) {
  exports.__wbg_set_reinit_hook(function () {
    registerPanicHook();
    instanceId++;
  });
}

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

// Lightweight Proxy for exported classes (Durable Objects, RPC).
// After a wasm reinit, existing class instances hold stale wasm pointers.
// The reinit hook increments instanceId; the Proxy detects the mismatch
// and transparently re-constructs the instance from its stored args.
const classProxyHooks = {
  construct(ctor, args, newTarget) {
    const holder = {
      instance: Reflect.construct(ctor, args, newTarget),
      instanceId,
      ctor,
      args,
      newTarget,
    };
    return new Proxy(holder, instanceProxyHooks);
  },
};

const instanceProxyHooks = {
  get(target, prop, receiver) {
    if (target.instanceId !== instanceId) {
      target.instance = Reflect.construct(target.ctor, target.args, target.newTarget);
      target.instanceId = instanceId;
    }
    const val = Reflect.get(target.instance, prop, receiver);
    if (typeof val !== "function") return val;
    return function (...fnArgs) {
      return val.apply(target.instance, fnArgs);
    };
  },
  set: (target, prop, value, receiver) =>
    Reflect.set(target.instance, prop, value, receiver),
  has: (target, prop) => Reflect.has(target.instance, prop),
  deleteProperty: (target, prop) =>
    Reflect.deleteProperty(target.instance, prop),
  getPrototypeOf: (target) => Reflect.getPrototypeOf(target.instance),
  setPrototypeOf: (target, proto) =>
    Reflect.setPrototypeOf(target.instance, proto),
  isExtensible: (target) => Reflect.isExtensible(target.instance),
  preventExtensions: (target) => Reflect.preventExtensions(target.instance),
  getOwnPropertyDescriptor: (target, prop) =>
    Reflect.getOwnPropertyDescriptor(target.instance, prop),
  defineProperty: (target, prop, descriptor) =>
    Reflect.defineProperty(target.instance, prop, descriptor),
  ownKeys: (target) => Reflect.ownKeys(target.instance),
};

export default Entrypoint;
