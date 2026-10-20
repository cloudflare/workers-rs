#!/usr/bin/env node

/**
 * Benchmark runner: the same routes on plain JS, `worker` and `ts-gen-worker`.
 *
 * One Miniflare per variant, all built with the same options and alive at once. Requests are
 * dispatched in-process (`dispatchFetch`) and timed with `performance.now()` from dispatch to
 * the last body byte. Variants are interleaved request by request, rotating the order each
 * round, so machine drift lands on all of them alike. Each repetition waits for a 1-minute load
 * average below 2, measures cold start, starts fresh instances, sends `--prewarm` rounds of
 * `/hello` to all of them (the Node side of `dispatchFetch` takes hundreds of requests to reach
 * a steady state), then for every route discards the first `--warmup` requests per variant
 * and keeps the next `--requests`.
 *
 * Build first: `benchmark/build.sh` (or `chomp build:bench`) from the workspace root.
 */

import { Miniflare } from 'miniflare';
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { loadavg } from 'node:os';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const here = dirname(fileURLToPath(import.meta.url));

const { values: args } = parseArgs({
  options: {
    requests: { type: 'string', default: '2000' },
    warmup: { type: 'string', default: '50' },
    reps: { type: 'string', default: '3' },
    cold: { type: 'string', default: '10' },
    prewarm: { type: 'string', default: '500' },
    routes: { type: 'string' },
    out: { type: 'string' },
  },
});
const REQUESTS = Number(args.requests);
const WARMUP = Number(args.warmup);
const REPS = Number(args.reps);
const COLD = Number(args.cold);
const PREWARM = Number(args.prewarm);

const VARIANTS = [
  { name: 'js', dir: join(here, 'js'), main: 'index.js' },
  { name: 'worker', dir: join(here, 'build'), main: 'index.js' },
  { name: 'worker (no shim)', dir: join(here, 'build-noshim'), main: 'entry.js' },
  { name: 'ts-gen-worker', dir: join(here, '../ts-gen-worker/benchmark/build'), main: 'index.js' },
];

function collectModules(root) {
  const modules = {};
  (function walk(dir) {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      const path = join(dir, entry.name);
      const name = relative(root, path);
      if (entry.isDirectory()) walk(path);
      else if (/\.m?js$/.test(name)) modules[name] = { type: 'esm', contents: readFileSync(path, 'utf8') };
      else if (name.endsWith('.wasm')) modules[name] = { type: 'wasm', contents: new Uint8Array(readFileSync(path)) };
    }
  })(root);
  return modules;
}

for (const variant of VARIANTS) variant.modules = collectModules(variant.dir);

function miniflare(variant) {
  return new Miniflare({
    workers: [
      {
        config: {
          name: 'bench',
          compatibilityDate: '2026-03-17',
          manifest: { mainModule: variant.main, modules: variant.modules },
          env: {
            KV: { type: 'kv', id: 'KV' },
            SERVICE: { type: 'worker', worker: 'bench' },
          },
        },
      },
    ],
  });
}

// Deterministic payloads: 16 KB of ASCII for /echo, ~2 KB of JSON for /json.
const ECHO = Array.from({ length: 16384 }, (_, i) => String.fromCharCode(97 + ((i * 7) % 26))).join('');
const PAYLOAD = {
  id: 41,
  name: 'benchmark payload',
  active: true,
  score: 98.25,
  tags: ['alpha', 'beta', 'gamma', 'delta', 'epsilon', 'zeta', 'eta', 'theta'],
  items: Array.from({ length: 28 }, (_, i) => ({ id: i, label: `item-${i}-label`, qty: i % 7, price: 1.5 + i })),
};
const PAYLOAD_JSON = JSON.stringify(PAYLOAD);
const HEADERS = Object.fromEntries([1, 2, 3, 4, 5].map((i) => [`x-h${i}`, `request-value-${i}`]));

const decode = (body) => new TextDecoder().decode(body);

const HELLO = { name: 'hello', path: '/hello', check: (body) => decode(body) === 'hello' };
const ROUTES = [
  HELLO,
  {
    name: 'headers',
    path: '/headers',
    init: { headers: HEADERS },
    check: (_, res) => [1, 2, 3, 4, 5].every((i) => res.headers.get(`x-o${i}`) === HEADERS[`x-h${i}`]),
  },
  { name: 'echo', path: '/echo', init: { method: 'POST', body: ECHO }, check: (body) => decode(body) === ECHO },
  {
    name: 'json',
    path: '/json',
    init: { method: 'POST', body: PAYLOAD_JSON, headers: { 'content-type': 'application/json' } },
    check: (body) => {
      const out = JSON.parse(decode(body));
      return out.id === PAYLOAD.id + 1 && JSON.stringify({ ...out, id: PAYLOAD.id }) === PAYLOAD_JSON;
    },
  },
  { name: 'kv', path: '/kv', check: (body) => decode(body) === 'v'.repeat(1024) },
  { name: 'stream', path: '/stream', check: (body) => body.byteLength === 1024 * 1024 },
  { name: 'fanout', path: '/fanout', check: (body) => decode(body) === '50' },
].filter((route) => !args.routes || args.routes.split(',').includes(route.name));

async function hit(mf, route, variant) {
  const t0 = performance.now();
  const res = await mf.dispatchFetch(`http://bench${route.path}`, route.init);
  const body = await res.arrayBuffer();
  const ms = performance.now() - t0;
  if (res.status !== 200 || !route.check(body, res)) {
    throw new Error(`${variant.name} ${route.path}: ${res.status} ${decode(body).slice(0, 200)}`);
  }
  return ms;
}

async function quiet() {
  for (let waited = 0; ; waited += 10) {
    const load = loadavg()[0];
    if (load < 2) return load;
    if (waited >= 600) throw new Error(`load average ${load.toFixed(2)} stayed >= 2 for 10 minutes`);
    console.log(`  load ${load.toFixed(2)} >= 2, waiting`);
    await new Promise((resolve) => setTimeout(resolve, 10_000));
  }
}

const percentile = (sorted, q) => sorted[Math.min(sorted.length - 1, Math.ceil(q * sorted.length) - 1)];
const median = (xs) => [...xs].sort((a, b) => a - b)[Math.floor(xs.length / 2)];

function stats(samples) {
  const sorted = [...samples].sort((a, b) => a - b);
  const mean = samples.reduce((a, b) => a + b, 0) / samples.length;
  return {
    n: samples.length,
    p50: percentile(sorted, 0.5),
    p90: percentile(sorted, 0.9),
    p99: percentile(sorted, 0.99),
    mean,
    // Sequential: one request in flight, so throughput is the inverse of the mean latency.
    rps: 1000 / mean,
  };
}

// Fresh instance per sample: `new Miniflare` to first response (process start, module
// evaluation, first request), and `ready` to first response (the first request alone).
async function coldStart() {
  const total = VARIANTS.map(() => []);
  const first = VARIANTS.map(() => []);
  // The first instance in a process also pays Miniflare's own one-time setup.
  const discard = miniflare(VARIANTS[0]);
  await hit(discard, HELLO, VARIANTS[0]);
  await discard.dispose();
  for (let i = 0; i < COLD; i++) {
    for (let k = 0; k < VARIANTS.length; k++) {
      const v = (i + k) % VARIANTS.length;
      const t0 = performance.now();
      const mf = miniflare(VARIANTS[v]);
      await mf.ready;
      const t1 = performance.now();
      await hit(mf, HELLO, VARIANTS[v]);
      const t2 = performance.now();
      await mf.dispose();
      total[v].push(t2 - t0);
      first[v].push(t2 - t1);
    }
  }
  return VARIANTS.map((_, v) => ({ total: median(total[v]), first: median(first[v]) }));
}

async function repetition(rep) {
  const loadBefore = await quiet();
  console.log(`rep ${rep + 1}/${REPS}: load ${loadBefore.toFixed(2)}`);
  const cold = COLD > 0 ? await coldStart() : [];
  const instances = [];
  for (const variant of VARIANTS) {
    const mf = miniflare(variant);
    await mf.ready;
    instances.push(mf);
  }
  const routes = {};
  try {
    for (let i = 0; i < PREWARM; i++) {
      for (let v = 0; v < VARIANTS.length; v++) await hit(instances[v], HELLO, VARIANTS[v]);
    }
    for (const route of ROUTES) {
      const samples = VARIANTS.map(() => []);
      for (let i = 0; i < WARMUP + REQUESTS; i++) {
        for (let k = 0; k < VARIANTS.length; k++) {
          const v = (i + k) % VARIANTS.length;
          const ms = await hit(instances[v], route, VARIANTS[v]);
          if (i >= WARMUP) samples[v].push(ms);
        }
      }
      routes[route.name] = samples.map((xs) => ({ ...stats(xs), samples: xs }));
      const line = VARIANTS.map((variant, v) => `${variant.name} ${routes[route.name][v].p50.toFixed(3)}`).join('  ');
      console.log(`  ${route.name.padEnd(8)} p50 ms: ${line}`);
    }
  } finally {
    await Promise.all(instances.map((mf) => mf.dispose()));
  }
  return { loadBefore, loadAfter: loadavg()[0], cold, routes };
}

function sizes(variant) {
  const wasm = Object.entries(variant.modules).find(([name]) => name.endsWith('.wasm'));
  // `worker/shim.mjs` is worker-build's re-export alias of `index.js`.
  const js = Object.keys(variant.modules).filter((name) => /\.m?js$/.test(name) && name !== 'worker/shim.mjs');
  const jsBytes = js.reduce((n, name) => n + statSync(join(variant.dir, name)).size, 0);
  return { wasm: wasm ? wasm[1].contents.byteLength : 0, js: jsBytes };
}

const results = [];
for (let rep = 0; rep < REPS; rep++) results.push(await repetition(rep));

const fmt = (ms) => ms.toFixed(3);
console.log(`\n${REQUESTS} requests per route per variant after ${WARMUP} warmup, ${REPS} repetitions;`);
console.log('each cell is the median over repetitions, with the p50 range over repetitions.\n');
console.log('| route | variant | p50 ms | p50 range | p90 ms | p99 ms | req/s |');
console.log('| --- | --- | ---: | ---: | ---: | ---: | ---: |');
for (const route of ROUTES) {
  VARIANTS.forEach((variant, v) => {
    const per = results.map((r) => r.routes[route.name][v]);
    const p50s = per.map((s) => s.p50);
    const cells = [
      fmt(median(p50s)),
      `${fmt(Math.min(...p50s))}–${fmt(Math.max(...p50s))}`,
      fmt(median(per.map((s) => s.p90))),
      fmt(median(per.map((s) => s.p99))),
      median(per.map((s) => s.rps)).toFixed(0),
    ];
    console.log(`| \`${route.path}\` | ${variant.name} | ${cells.join(' | ')} |`);
  });
}
if (COLD > 0) {
  console.log(`\n| variant | .wasm bytes | JS bytes | cold start ms (new Miniflare → response) | first request ms (ready → response) |`);
  console.log('| --- | ---: | ---: | ---: | ---: |');
  VARIANTS.forEach((variant, v) => {
    const { wasm, js } = sizes(variant);
    const total = results.map((r) => fmt(r.cold[v].total)).join(', ');
    const first = results.map((r) => fmt(r.cold[v].first)).join(', ');
    console.log(`| ${variant.name} | ${wasm} | ${js} | ${total} | ${first} |`);
  });
}
console.log(`\nload average before/after each repetition: ${results.map((r) => `${r.loadBefore.toFixed(2)}/${r.loadAfter.toFixed(2)}`).join(', ')}`);

const out = args.out ?? process.env.BENCH_RESULT;
if (out) {
  writeFileSync(out, JSON.stringify({ requests: REQUESTS, warmup: WARMUP, variants: VARIANTS.map((v) => v.name), results }));
  console.log(`results written to ${out}`);
}
