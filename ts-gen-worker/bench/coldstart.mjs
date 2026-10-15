// Cold start to first response: median over N fresh Miniflare instances of
// `performance.now()` around `dispatchFetch(path)`, each instance disposed after
// one request. Usage: node coldstart.mjs <build-dir> <main-module> [path] [N]
import { readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import { Miniflare } from "miniflare";

const [dir, mainModule, path = "/", runs = "10"] = process.argv.slice(2);

const modules = {};
function collect(d) {
  for (const entry of readdirSync(d, { withFileTypes: true })) {
    const path = join(d, entry.name);
    const name = relative(dir, path);
    if (entry.isDirectory()) collect(path);
    else if (/\.m?js$/.test(name)) modules[name] = { type: "esm", contents: readFileSync(path, "utf8") };
    else if (name.endsWith(".wasm")) modules[name] = { type: "wasm", contents: new Uint8Array(readFileSync(path)) };
  }
}
collect(dir);

const times = [];
for (let i = 0; i < Number(runs); i++) {
  const mf = new Miniflare({
    workers: [{ config: { name: "bench", compatibilityDate: "2026-03-17", manifest: { mainModule, modules } } }],
  });
  const t0 = performance.now();
  const res = await mf.dispatchFetch(`http://bench.test${path}`);
  const text = await res.text();
  times.push(performance.now() - t0);
  await mf.dispose();
  if (i === 0) console.log(`first response: ${res.status} ${JSON.stringify(text)}`);
}
times.sort((a, b) => a - b);
const median = times[Math.floor(times.length / 2)];
console.log(`runs=${times.length} median=${median.toFixed(1)}ms min=${times[0].toFixed(1)}ms max=${times.at(-1).toFixed(1)}ms`);
