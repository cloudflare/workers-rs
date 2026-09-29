// Per-fixture, per-stage pipeline time against a running worker.
//
// The pipeline runs `repeat` times inside each request so the request
// overhead (measured by the sniff stage at repeat=1 and subtracted) is
// amortised. Stages are cumulative: the delta column is that stage's cost.
//
//   node bench/bench.mjs [url] [repeat] [samples]
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const url = process.argv[2] ?? "http://localhost:8787";
const repeat = Number(process.argv[3] ?? 10);
const samples = Number(process.argv[4] ?? 5);
const dir = join(dirname(fileURLToPath(import.meta.url)), "fixtures");
const stages = ["sniff", "decode", "resize", "encode"];

const median = (xs) => xs.sort((a, b) => a - b)[xs.length >> 1];

async function run(body, headers, stage, n) {
  const times = [];
  let res;
  for (let i = 0; i < samples; i++) {
    const start = performance.now();
    res = await fetch(`${url}/?stage=${stage}&repeat=${n}`, { method: "POST", body, headers });
    await res.arrayBuffer();
    times.push(performance.now() - start);
    if (!res.ok) throw new Error(`${stage}: ${res.status} ${await res.text()}`);
  }
  return { ms: median(times), mem: Number(res.headers.get("x-wasm-memory-bytes")), info: res.headers.get("x-image-info") };
}

const rows = [];
for (const file of readdirSync(dir).sort()) {
  const bytes = readFileSync(join(dir, file));
  const ext = file.split(".").pop();
  const mime = { jpg: "image/jpeg", png: "image/png", webp: "image/webp", gif: "image/gif" }[ext];
  const dataUrl = `data:${mime};base64,${bytes.toString("base64")}`;
  const overhead = (await run(bytes, { "content-type": mime }, "sniff", 1)).ms;
  const b64Overhead = (await run(dataUrl, { "content-type": "text/plain" }, "sniff", 1)).ms;
  let prev = 0;
  for (const stage of stages) {
    const r = await run(bytes, { "content-type": mime }, stage, repeat);
    const ms = (r.ms - overhead) / repeat;
    rows.push({ file, stage, ms: ms.toFixed(1), delta: (ms - prev).toFixed(1), mem_mb: (r.mem / 2 ** 20).toFixed(0) });
    prev = ms;
  }
  const r = await run(dataUrl, { "content-type": "text/plain" }, "encode", repeat);
  const ms = (r.ms - b64Overhead) / repeat;
  rows.push({ file, stage: "encode+base64", ms: ms.toFixed(1), delta: (ms - prev).toFixed(1), mem_mb: (r.mem / 2 ** 20).toFixed(0) });
  const info = JSON.parse(r.info)[0];
  rows.push({ file, stage: `${info.width}x${info.height} -> ${info.out_width}x${info.out_height}, ${info.in_bytes} -> ${info.out_bytes} B` });
}
console.table(rows);
