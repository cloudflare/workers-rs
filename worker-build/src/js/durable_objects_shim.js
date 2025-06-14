import { DurableObject } from "cloudflare:workers";

const successfullyWrappedDONames = [];
const globalStorePrefix = "__DO_WRAPPED_"; // For globalThis storage

if (typeof __WORKER_BUILD_DO_NAMES__ !== 'undefined' && Array.isArray(__WORKER_BUILD_DO_NAMES__)) {
    __WORKER_BUILD_DO_NAMES__.forEach(className => {
        const OriginalClass = imports[className];
        if (typeof OriginalClass === 'function' && OriginalClass.prototype) {
            console.log(`[shim.js] Wrapping DO: ${className}`);
            successfullyWrappedDONames.push(className);
            const WrappedClass = class extends DurableObject {
                constructor(state, env) { super(state, env); this._inner = new OriginalClass(state, env); }
            };
            Object.getOwnPropertyNames(OriginalClass.prototype).forEach(methodName => {
                if (methodName !== 'constructor') {
                    if (typeof OriginalClass.prototype[methodName] === 'function') {
                        WrappedClass.prototype[methodName] = function(...args) { return this._inner[methodName](...args); };
                    }
                }
            });
            globalThis[`${globalStorePrefix}${className}`] = WrappedClass;
        } else {
            console.warn(`[shim.js] DO '${className}' (from __WORKER_BUILD_DO_NAMES__) not found/invalid in wasm imports.`);
        }
    });
}