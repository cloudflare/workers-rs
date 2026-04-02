// Simplified shim for panic=unwind builds.
//
// With panic=unwind, wasm-bindgen automatically handles:
//   - Termination detection via Wasm-level catch wrappers (generated when
//     __instance_terminated is present in the Wasm binary)
//   - Automatic instance reset via __wbg_termination_guard() at every export
//   - Instance ID tracking for stale object detection
//
// The only JS-side concern remaining is re-constructing Durable Object class
// instances after a Wasm reset, since the Workers runtime reuses the same
// JS object across requests.
//
// The Entrypoint class is exported directly (not proxied) because its
// prototype methods delegate to `exports.*`, which are live ESM bindings
// that automatically resolve to the new instance after __wbg_reset_state().

import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";

Error.stackTraceLimit = 100;

// Shared state object from the worker crate's inline_js snippet.  The object
// reference is grabbed once here (single Wasm hop at module load); after that
// every read of `reinitState.id` is a plain JS property access — no Wasm
// boundary crossing on the hot path.
const reinitState = exports.__worker_reinit_state();

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

export default Entrypoint;

// ---------------------------------------------------------------------------
// Durable Object class proxy
//
// After a Wasm reset, class instances from the previous instance become stale.
// The worker crate maintains a generation counter (`reinitState.id`) in a
// shared JS object that is bumped by a set_on_reinit hook after each reset.
// The proxy snapshots the counter at construction time and compares before
// each method call; a mismatch means the Wasm instance was reset and the DO
// must be re-constructed.
// ---------------------------------------------------------------------------

const instanceProxyHooks = {
  set: (target, prop, value, receiver) => Reflect.set(target.instance, prop, value, receiver),
  has: (target, prop) => Reflect.has(target.instance, prop),
  deleteProperty: (target, prop) => Reflect.deleteProperty(target.instance, prop),
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
    const target = {
      instance: Reflect.construct(ctor, args, newTarget),
      instanceId: reinitState.id,
      ctor,
      args,
      newTarget,
    };
    return new Proxy(target, {
      ...instanceProxyHooks,
      get(target, prop, receiver) {
        if (target.instanceId !== reinitState.id) {
          target.instance = Reflect.construct(target.ctor, target.args, target.newTarget);
          target.instanceId = reinitState.id;
        }
        const original = Reflect.get(target.instance, prop, receiver);
        if (typeof original !== 'function') return original;
        return new Proxy(original, {
          apply(fn, thisArg, argArray) {
            return fn.apply(target.instance, argArray);
          }
        });
      }
    });
  }
};
