import { describe, test, expect, afterAll } from "vitest";
import { Miniflare } from "miniflare";
import { readFileSync } from "fs";
import { resolve } from "path";

const wasmPath = resolve(__dirname, "signal_wasm", "signal.wasm");
const wasmBytes = readFileSync(wasmPath);

const mf = new Miniflare({
  modules: [
    {
      type: "ESModule",
      path: "entrypoint.mjs",
      contents: `
        import wasmModule from "./signal.wasm";

        export default {
          async fetch(request) {
            const instance = new WebAssembly.Instance(wasmModule);
            const memory = new DataView(instance.exports.memory.buffer);
            const signalAddr = instance.exports.__signal_address.value;
            const terminatedAddr = instance.exports.__terminated_address.value;
            const signal = memory.getInt32(signalAddr, true);
            const terminated = memory.getInt32(terminatedAddr, true);
            return Response.json({ signal, terminated });
          }
        }
      `,
    },
    {
      type: "CompiledWasm",
      path: "signal.wasm",
      contents: new Uint8Array(wasmBytes),
    },
  ],
});

afterAll(async () => {
  await mf.dispose();
});

describe("signal address initialization", () => {
  test("__signal_address is zeroed after wasm instantiation", async () => {
    const url = await mf.ready;
    const resp = await mf.dispatchFetch(url);
    expect(resp.status).toBe(200);

    const body = (await resp.json()) as { signal: number; terminated: number };
    expect(body.signal).toBe(0);
    expect(body.terminated).toBe(0);
  });
});
