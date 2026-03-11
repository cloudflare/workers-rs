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

const liveInstances = new Set();
const cleanup = typeof FinalizationRegistry !== "undefined"
  ? new FinalizationRegistry((holder) => liveInstances.delete(holder))
  : { register() {}, unregister() {} };

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
// so we can re-register the panic hook and reconstruct all live DO/RPC
// class instances on the fresh wasm module.
if (exports.__wbg_set_reinit_hook) {
  exports.__wbg_set_reinit_hook(function () {
    registerPanicHook();
    for (const holder of liveInstances) {
      holder.instance = Reflect.construct(holder.ctor, holder.args, holder.newTarget);
    }
  });
}

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

// Lightweight Proxy for exported classes (Durable Objects, RPC).
// After a wasm reinit, existing class instances hold stale wasm pointers.
// The reinit hook eagerly reconstructs all tracked instances so the Proxy
// can delegate without any per-access staleness checks.
const classProxyHooks = {
  construct(ctor, args, newTarget) {
    const holder = {
      instance: Reflect.construct(ctor, args, newTarget),
      ctor,
      args,
      newTarget,
    };
    liveInstances.add(holder);
    const proxy = new Proxy(holder, instanceProxyHooks);
    cleanup.register(proxy, holder);
    return proxy;
  },
};

const instanceProxyHooks = {
  get(target, prop, receiver) {
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
