import { WorkerEntrypoint } from "cloudflare:workers";
import * as exports from "./index.js";
export * from "./index.js";

let panicError = null;
Error.stackTraceLimit = 100;

function registerPanicHook() {
  exports.setPanicHook(function (message) {
    panicError = new Error("Critical Rust panic: " + message);
    console.error(panicError);
  });
}

registerPanicHook();

function checkReinitialize() {
  if (panicError) {
    console.log("Reinitializing Wasm application");
    exports.__wbg_reset_state();
    panicError = null;
    registerPanicHook();
    for (const instance of instances) {
      const newInstance = Reflect.construct(instance.target, instance.args, instance.newTarget);
      instance.instance = newInstance;
    }
  }
}

export default class Entrypoint extends WorkerEntrypoint {
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

const instances = [];
const classProxyHooks = {
  construct(target, args, newTarget) {
    const instance = {
      instance: Reflect.construct(target, args, newTarget),
      target,
      args,
      newTarget
    };
    instances.push(instance);
    return new Proxy(instance, {
      get(target, prop, receiver) {
        return Reflect.get(target.instance, prop, receiver);
      },
      
      set(target, prop, value, receiver) {
        return Reflect.set(target.instance, prop, value, receiver);
      }
    });
  }
};
