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

const modules: Record<string, Module> = {};
collect("", modules);

export const manifest = { mainModule: "index.js", modules };

export const publicDir = fileURLToPath(new URL("../public", import.meta.url));
