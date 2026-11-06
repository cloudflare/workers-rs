import { DurableObject, WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";

// An Emscripten abort (a Rust panic with nowhere to unwind, or a trap) leaves the
// module unable to run callbacks, so in-flight and later calls would hang until the
// runtime cancels them. Instead they fail with the abort as the cause.
let aborted = null;
const pending = new Set();

function markAborted(e) {
  if (aborted) return;
  aborted = e;
  for (const reject of pending) reject(abortedError());
  pending.clear();
}

function abortedError() {
  return new Error("The Wasm module aborted and cannot handle further calls", { cause: aborted });
}

globalThis[Symbol.for("workers-rs.abort")] = (what) =>
  markAborted(new WebAssembly.RuntimeError(`Aborted(${what ?? ""})`));

function markIfTrap(e) {
  if (e instanceof WebAssembly.RuntimeError) markAborted(e);
}

function guard(fn) {
  return function (...args) {
    if (aborted) throw abortedError();
    let result;
    try {
      result = fn.apply(this, args);
    } catch (e) {
      markIfTrap(e);
      throw e;
    }
    if (!(result instanceof Promise)) return result;
    return new Promise((resolve, reject) => {
      pending.add(reject);
      result.then(resolve, (e) => {
        markIfTrap(e);
        reject(e);
      }).finally(() => pending.delete(reject));
    });
  };
}

function guardPrototype(proto) {
  for (const key of Object.getOwnPropertyNames(proto)) {
    if (key === "constructor") continue;
    const desc = Object.getOwnPropertyDescriptor(proto, key);
    if (typeof desc.value === "function") {
      Object.defineProperty(proto, key, { ...desc, value: guard(desc.value) });
    }
  }
}

class Entrypoint extends WorkerEntrypoint {}

$HANDLERS

guardPrototype(Entrypoint.prototype);

export default Entrypoint;
