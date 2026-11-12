import { readdirSync, readFileSync } from "node:fs";

// The build output is a module graph: the entry, the wasm-bindgen module it
// imports, and that module's snippets and wasm.
const modules: Record<
  string,
  | { type: "esm"; contents: string }
  | { type: "wasm"; contents: Uint8Array }
> = {};
for (const file of readdirSync("./build", { recursive: true }) as string[]) {
  const name = `build/${file.replaceAll("\\", "/")}`;
  if (/\.m?js$/.test(name)) {
    modules[name] = { type: "esm", contents: readFileSync(name, "utf8") };
  } else if (name.endsWith(".wasm")) {
    modules[name] = { type: "wasm", contents: new Uint8Array(readFileSync(name)) };
  }
}

export const manifest = { mainModule: "build/index.js", modules };
