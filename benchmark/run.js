#!/usr/bin/env node

/**
 * Benchmark runner for workers-rs
 *
 * This script runs performance benchmarks against the worker server.
 * It measures the time taken to complete a benchmark that makes 10 parallel
 * sub-requests, each streaming 1MB of data internally.
 */

import { Miniflare } from 'miniflare';
import { readFileSync, writeFileSync } from 'node:fs';

async function runBenchmark() {
  console.log('🚀 Starting workers-rs benchmark suite\n');

  // Initialize Miniflare instance with the compiled worker
  console.log('📦 Initializing Miniflare...');
  const mf = new Miniflare({
    workers: [
      {
        config: {
          name: 'benchmark',
          type: 'worker',
          compatibilityDate: '2025-01-06',
          manifest: {
            mainModule: 'build/index.js',
            modules: {
              'build/index.js': {
                type: 'esm',
                contents: readFileSync('./build/index.js', 'utf8'),
              },
              'build/index_bg.wasm': {
                type: 'wasm',
                contents: new Uint8Array(readFileSync('./build/index_bg.wasm')),
              },
            },
          },
        },
        dev: {
          outboundService: { type: 'worker', worker: 'benchmark' },
        },
      }
    ]
  });

  const mfUrl = await mf.ready;
  console.log(`✅ Miniflare ready at ${mfUrl}\n`);

  // Run warmup requests
  console.log('🔥 Running warmup requests...');
  for (let i = 0; i < 3; i++) {
    await mf.dispatchFetch(`${mfUrl}benchmark`);
  }
  console.log('✅ Warmup complete\n');

  // Run benchmark iterations
  const iterations = 20;
  const results = [];

  console.log(`📊 Running ${iterations} benchmark iterations...\n`);

  for (let i = 0; i < iterations; i++) {
    const iterStart = Date.now();
    const response = await mf.dispatchFetch(`${mfUrl}benchmark`);
    const iterEnd = Date.now();

    const nodeJsDuration = iterEnd - iterStart;
    const result = await response.json();

    if (!result.success) {
      console.error(`❌ Iteration ${i + 1} failed:`, result.errors);
      await mf.dispose();
      process.exit(1);
    }

    results.push({
      iteration: i + 1,
      nodeJsDuration,
      workerDuration: result.duration_ms,
      totalBytes: result.total_bytes,
      numRequests: result.num_requests,
    });

    console.log(`  Iteration ${i + 1}:`);
    console.log(`    Node.js end-to-end time: ${nodeJsDuration}ms`);
    console.log(`    Worker internal time:    ${result.duration_ms}ms`);
    console.log(`    Data transferred:        ${(result.total_bytes / (1024 * 1024)).toFixed(2)}MB`);
    console.log(`    Sub-requests:            ${result.num_requests}`);
    console.log();
  }

  // Calculate statistics
  const nodeJsDurations = results.map(r => r.nodeJsDuration);
  const workerDurations = results.map(r => r.workerDuration);

  const avgNodeJs = nodeJsDurations.reduce((a, b) => a + b, 0) / iterations;
  const avgWorker = workerDurations.reduce((a, b) => a + b, 0) / iterations;

  const minNodeJs = Math.min(...nodeJsDurations);
  const maxNodeJs = Math.max(...nodeJsDurations);
  const minWorker = Math.min(...workerDurations);
  const maxWorker = Math.max(...workerDurations);

  // Print summary
  console.log('━'.repeat(60));
  console.log('📈 BENCHMARK SUMMARY');
  console.log('━'.repeat(60));
  console.log();
  console.log('Node.js End-to-End Time:');
  console.log(`  Average: ${avgNodeJs.toFixed(2)}ms`);
  console.log(`  Min:     ${minNodeJs.toFixed(2)}ms`);
  console.log(`  Max:     ${maxNodeJs.toFixed(2)}ms`);
  console.log();
  console.log('Worker Internal Time:');
  console.log(`  Average: ${avgWorker.toFixed(2)}ms`);
  console.log(`  Min:     ${minWorker.toFixed(2)}ms`);
  console.log(`  Max:     ${maxWorker.toFixed(2)}ms`);
  console.log();
  console.log('Benchmark Configuration:');
  console.log(`  Parallel sub-requests:    10`);
  console.log(`  Data per sub-request:     1MB`);
  console.log(`  Total data per iteration: 10MB`);
  console.log(`  Iterations:               ${iterations}`);
  console.log();
  console.log('━'.repeat(60));

  // Calculate throughput
  const totalBytes = results[0].totalBytes;
  const throughputMbps = (totalBytes * 8 / (avgWorker / 1000)) / (1024 * 1024);
  console.log(`🚀 Average throughput: ${throughputMbps.toFixed(2)} Mbps`);
  console.log('━'.repeat(60));

  // Write results JSON if BENCH_RESULT env is set
  const resultPath = process.env.BENCH_RESULT;
  if (resultPath) {
    writeFileSync(resultPath, JSON.stringify(results, null, 2));
    console.log(`\n📁 Results written to ${resultPath}`);
  }

  // Cleanup
  await mf.dispose();
  console.log('\n✅ Benchmark complete!');
}

runBenchmark().catch((error) => {
  console.error('❌ Benchmark failed:', error);
  process.exit(1);
});
