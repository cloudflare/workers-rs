import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

// wasm-bindgen --target module emits index.js, index_bg.wasm and, for
// `inline_js` snippets, snippets/<crate>-<hash>/*.js next to them. The
// manifest mirrors the directory so relative imports resolve unchanged.
const dir = new URL("../build/", import.meta.url);

type Module =
  | { type: "esm"; contents: string }
  | { type: "wasm"; contents: Uint8Array };

function collect(prefix: string, modules: Record<string, Module>) {
  for (const entry of readdirSync(new URL(prefix, dir), { withFileTypes: true })) {
    const name = `${prefix}${entry.name}`;
    if (entry.isDirectory()) {
      collect(`${name}/`, modules);
    } else if (name.endsWith(".js")) {
      modules[name] = { type: "esm", contents: readFileSync(new URL(name, dir), "utf8") };
    } else if (name.endsWith(".wasm")) {
      modules[name] = {
        type: "wasm",
        contents: new Uint8Array(readFileSync(new URL(name, dir))),
      };
    }
  }
}

// Test-harness stand-in, not part of the crate: after a trap the glue
// reinstantiates the Wasm module and then rejects every method call on a
// Durable Object instance the runtime constructed in the old instance
// ("Invalid stale object from previous Wasm instance"). Each exported class is
// wrapped so that error re-runs the constructor with the saved arguments and
// retries the call. The runtime reads handler methods once at construction, so
// the cached functions resolve the instance per call rather than binding it.
// Replaced by wasm-bindgen's `#[wasm_bindgen(reinit = "reconstruct")]` once the
// workspace pin includes it: `#[durable_object]` emits the attribute and this
// wrapper goes away; see parity/errors.md "Session 6".
function reinitShim(index: string): string {
  const classes = [...index.matchAll(/^export class (\w+)/gm)].map((m) => m[1]);
  return `import * as exports from "./index.js";
export { default } from "./index.js";

const STALE = "Invalid stale object from previous Wasm instance";

function reconstructing(ctor) {
  return new Proxy(ctor, {
    construct(target, args, newTarget) {
      let instance = Reflect.construct(target, args, newTarget);
      const call = (prop, callArgs) => {
        try {
          return Reflect.apply(Reflect.get(instance, prop), instance, callArgs);
        } catch (e) {
          if (!(e instanceof Error) || e.message !== STALE) throw e;
          instance = Reflect.construct(target, args, newTarget);
          return Reflect.apply(Reflect.get(instance, prop), instance, callArgs);
        }
      };
      return new Proxy(instance, {
        get(_, prop) {
          const value = Reflect.get(instance, prop);
          if (typeof value !== "function") return value;
          return new Proxy(value, { apply: (_fn, _this, callArgs) => call(prop, callArgs) });
        },
        set: (_, prop, value) => Reflect.set(instance, prop, value),
        has: (_, prop) => Reflect.has(instance, prop),
        getPrototypeOf: () => Reflect.getPrototypeOf(instance),
      });
    },
  });
}
${classes.map((c) => `export const ${c} = reconstructing(exports.${c});`).join("\n")}
`;
}

const modules: Record<string, Module> = {};
collect("", modules);
const index = modules["index.js"];
if (index.type === "esm") {
  modules["shim.js"] = { type: "esm", contents: reinitShim(index.contents) };
}

export const manifest = { mainModule: "shim.js", modules };

export const publicDir = fileURLToPath(new URL("../public", import.meta.url));
