#!/usr/bin/env node

/**
 * Converts workers-rs benchmark results into CodSpeed walltime format.
 * Based on the codspeed-rust 4.1.0 result schema.
 *
 * Usage: node codspeed-convert.js <input.json>
 *
 * Writes results to $CODSPEED_PROFILE_FOLDER/results/<pid>.json
 * or ./codspeed-results/<pid>.json as fallback.
 */

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const IQR_OUTLIER_FACTOR = 1.5;
const STDEV_OUTLIER_FACTOR = 3.0;

function quantile(sorted, q) {
  const pos = (sorted.length - 1) * q;
  const base = Math.floor(pos);
  const rest = pos - base;
  if (sorted[base + 1] !== undefined) {
    return sorted[base] + rest * (sorted[base + 1] - sorted[base]);
  }
  return sorted[base];
}

function computeStats(name, uri, timesMs) {
  // Convert ms to ns
  const timesNs = timesMs.map(t => t * 1_000_000);
  const sorted = [...timesNs].sort((a, b) => a - b);
  const rounds = timesNs.length;

  const mean = timesNs.reduce((a, b) => a + b, 0) / rounds;
  const stdev = rounds < 2 ? 0 : Math.sqrt(
    timesNs.reduce((sum, t) => sum + (t - mean) ** 2, 0) / (rounds - 1)
  );

  const q1 = quantile(sorted, 0.25);
  const median = quantile(sorted, 0.5);
  const q3 = quantile(sorted, 0.75);
  const iqr = q3 - q1;

  const iqrOutliers = timesNs.filter(
    t => t < q1 - IQR_OUTLIER_FACTOR * iqr || t > q3 + IQR_OUTLIER_FACTOR * iqr
  ).length;
  const stdevOutliers = timesNs.filter(
    t => t < mean - STDEV_OUTLIER_FACTOR * stdev || t > mean + STDEV_OUTLIER_FACTOR * stdev
  ).length;

  return {
    name,
    uri,
    config: {},
    stats: {
      min_ns: sorted[0],
      max_ns: sorted[sorted.length - 1],
      mean_ns: mean,
      stdev_ns: stdev,
      q1_ns: q1,
      median_ns: median,
      q3_ns: q3,
      rounds,
      total_time: timesNs.reduce((a, b) => a + b, 0) / 1_000_000_000,
      iqr_outlier_rounds: iqrOutliers,
      stdev_outlier_rounds: stdevOutliers,
      iter_per_round: 1,
      warmup_iters: 3,
    },
  };
}

const inputPath = process.argv[2];
if (!inputPath) {
  console.error('Usage: node codspeed-convert.js <input.json>');
  process.exit(1);
}

const results = JSON.parse(readFileSync(inputPath, 'utf-8'));

const benchmarks = [
  computeStats(
    'worker_internal_time',
    'benchmark/src/lib.rs::handle_benchmark::worker_internal_time',
    results.map(r => r.workerDuration)
  ),
  computeStats(
    'e2e_time',
    'benchmark/run.js::runBenchmark::e2e_time',
    results.map(r => r.nodeJsDuration)
  ),
];

const output = {
  creator: {
    name: 'codspeed-node',
    version: '4.1.0',
    pid: process.pid,
  },
  instrument: { type: 'walltime' },
  benchmarks,
};

const profileFolder = process.env.CODSPEED_PROFILE_FOLDER || 'codspeed-results';
const resultsDir = join(profileFolder, 'results');
mkdirSync(resultsDir, { recursive: true });

const outputPath = join(resultsDir, `${process.pid}.json`);
writeFileSync(outputPath, JSON.stringify(output, null, 2));
console.log(`CodSpeed results written to ${outputPath}`);
