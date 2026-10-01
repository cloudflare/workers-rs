// Interleaved A/B of the decode stage between two running variants.
//   node bench/ab.mjs <urlA> <urlB> [file] [repeat] [rounds]
import { readFileSync } from "node:fs";
const [a, b, file = "bench/fixtures/12mp.jpg", repeat = 20, rounds = 15] = process.argv.slice(2);
const bytes = readFileSync(file);
const headers = { "content-type": "image/jpeg" };
async function one(url, stage, n) {
  const t = performance.now();
  const r = await fetch(`${url}/?stage=${stage}&repeat=${n}`, { method: "POST", body: bytes, headers });
  await r.arrayBuffer();
  return performance.now() - t;
}
const med = (xs) => xs.sort((x, y) => x - y)[xs.length >> 1];
const res = { [a]: [], [b]: [] };
for (const u of [a, b]) for (let i = 0; i < 3; i++) await one(u, "decode", 2); // warm
for (let i = 0; i < rounds; i++) for (const u of i % 2 ? [b, a] : [a, b]) {
  const o = await one(u, "sniff", 1);
  res[u].push((await one(u, "decode", repeat) - o) / repeat);
}
for (const u of [a, b]) console.log(u, `decode median ${med(res[u]).toFixed(1)} ms  min ${Math.min(...res[u]).toFixed(1)}`);
