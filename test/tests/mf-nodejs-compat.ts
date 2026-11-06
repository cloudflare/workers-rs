import { readFileSync } from "node:fs";
import { Miniflare } from "miniflare";

// A second instance of the test worker on a compatibility date from which
// `nodejs_compat` (and with it `enable_nodejs_global_timers`) is on by default,
// so `setTimeout` returns a Node.js `Timeout` object instead of a number.
// No bindings: only routes that need none should be exercised here.
const manifest = {
  mainModule: "build/index.js",
  modules: {
    "build/index.js": {
      type: "esm" as const,
      contents: readFileSync("./build/index.js", "utf8"),
    },
    "build/index_bg.wasm": {
      type: "wasm" as const,
      contents: new Uint8Array(readFileSync("./build/index_bg.wasm")),
    },
  },
};

export const mf = new Miniflare({
  workers: [
    {
      config: {
        name: "test-nodejs-compat",
        compatibilityDate: "2026-08-04",
        manifest,
      },
    },
  ],
});

export const mfUrl = await mf.ready;
