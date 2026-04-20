#!/usr/bin/env node
//
// Micro-benchmark for the R2 multi-get paths in the sandbox worker.
//
// Exercises each concurrency strategy N times under Miniflare and reports the
// median `elapsed_ms` reported by the worker itself, so we can compare:
//
//   parallel     — hand-rolled `Promise.all` over `future_to_promise`.
//   chunked      — `futures_util::stream::buffer_unordered(32)`.
//   chunked-js   — chunks of 32 awaited via `js_sys::futures::join_all`.
//   join         — single `js_sys::futures::join_all` over all 512 keys.
//
// Assumes the sandbox has been built (`test/build/index.js` exists and is
// up-to-date). Run with `node test/r2-perf.mjs` from the repo root.

import { Miniflare, Response, createFetchMock } from "miniflare";
import { writeFileSync } from "node:fs";

const ITERATIONS = Number(process.env.ITERATIONS ?? 10);
const WARMUP = Number(process.env.WARMUP ?? 2);
const MODES = [
  { path: "r2/get-many-parallel", label: "parallel" },
  { path: "r2/get-many-chunked", label: "chunked" },
  { path: "r2/get-many-chunked-js", label: "chunked-js" },
  { path: "r2/get-many-join", label: "join" },
];

// Same shape as test/tests/mf.ts — kept minimal because r2-perf only needs the
// R2 buckets; the rest of the sandbox bindings are tolerated as-is because the
// worker only reads `PUT_BUCKET` on these routes.
const mf = new Miniflare({
  d1Persist: false,
  kvPersist: false,
  r2Persist: false,
  cachePersist: false,
  workers: [
    {
      scriptPath: "./test/build/index.js",
      compatibilityDate: "2025-07-24",
      cache: true,
      d1Databases: ["DB"],
      modules: true,
      modulesRules: [
        { type: "CompiledWasm", include: ["**/*.wasm"], fallthrough: true },
      ],
      bindings: {
        EXAMPLE_SECRET: "example",
        SOME_SECRET: "secret!",
        SOME_VARIABLE: "some value",
        SOME_OBJECT_VARIABLE: { foo: 42, bar: "string" },
      },
      durableObjects: {
        COUNTER: "Counter",
        PUT_RAW_TEST_OBJECT: "PutRawTestObject",
        AUTO: "AutoResponseObject",
        MY_CLASS: "MyClass",
        SQL_COUNTER: { className: "SqlCounter", useSQLite: true },
        SQL_ITERATOR: { className: "SqlIterator", useSQLite: true },
      },
      kvNamespaces: ["SOME_NAMESPACE", "FILE_SIZES", "TEST"],
      serviceBindings: {
        async remote() {
          return new Response("hello world");
        },
      },
      r2Buckets: ["EMPTY_BUCKET", "PUT_BUCKET", "SEEDED_BUCKET", "DELETE_BUCKET"],
      queueConsumers: { my_queue: { maxBatchTimeout: 1 } },
      queueProducers: ["my_queue", "my_queue"],
      fetchMock: createFetchMock(),
      secretsStoreSecrets: {
        SECRETS: { store_id: "SECRET_STORE", secret_name: "secret-name" },
        MISSING_SECRET: {
          store_id: "SECRET_STORE_MISSING",
          secret_name: "missing-secret",
        },
      },
      wrappedBindings: {
        HTTP_ANALYTICS: { scriptName: "mini-analytics-engine" },
      },
      ratelimits: {
        TEST_RATE_LIMITER: { simple: { limit: 10, period: 60 } },
      },
    },
    {
      name: "mini-analytics-engine",
      modules: true,
      script: `export default function (env) {
        return {
          writeDataPoint(data) {
            console.log(data)
          }
        }
      }`,
    },
  ],
});

await (await mf.getSecretsStoreSecretAPI("SECRETS"))().create("secret value");

const mfUrl = await mf.ready;
console.log(`✅ Miniflare ready at ${mfUrl}`);
console.log(`   iterations=${ITERATIONS}  warmup=${WARMUP}\n`);

const results = {};

for (const mode of MODES) {
  // Warmup — first hit pays the one-shot R2 seeding cost in the worker.
  for (let i = 0; i < WARMUP; i++) {
    const resp = await mf.dispatchFetch(`${mfUrl}${mode.path}`);
    if (resp.status !== 200) {
      console.error(`❌ ${mode.label} warmup failed: ${resp.status}`);
      await mf.dispose();
      process.exit(1);
    }
    await resp.json();
  }

  const samples = [];
  for (let i = 0; i < ITERATIONS; i++) {
    const resp = await mf.dispatchFetch(`${mfUrl}${mode.path}`);
    if (resp.status !== 200) {
      console.error(`❌ ${mode.label} iter ${i} failed: ${resp.status}`);
      await mf.dispose();
      process.exit(1);
    }
    const body = await resp.json();
    samples.push(body.elapsed_ms);
  }

  samples.sort((a, b) => a - b);
  const median = samples[Math.floor(samples.length / 2)];
  const min = samples[0];
  const max = samples[samples.length - 1];
  const mean = samples.reduce((a, b) => a + b, 0) / samples.length;

  results[mode.label] = { samples, median, min, max, mean };
  console.log(
    `${mode.label.padEnd(12)} median=${median}ms  mean=${mean.toFixed(1)}ms  min=${min}ms  max=${max}ms  samples=[${samples.join(", ")}]`,
  );
}

console.log();
console.log("━".repeat(60));
console.log("Relative to `parallel` (hand-rolled Promise.all):");
const baseline = results.parallel.median;
for (const mode of MODES) {
  const r = results[mode.label];
  const ratio = r.median / baseline;
  console.log(`  ${mode.label.padEnd(12)} ${ratio.toFixed(2)}× (${r.median}ms)`);
}
console.log("━".repeat(60));

const out = process.env.PERF_RESULT;
if (out) {
  writeFileSync(out, JSON.stringify(results, null, 2));
  console.log(`\n📁 Results written to ${out}`);
}

await mf.dispose();
