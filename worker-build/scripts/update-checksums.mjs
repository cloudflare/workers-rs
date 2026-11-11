#!/usr/bin/env node
// Regenerates worker-build/checksums.txt, the sha256 of every archive
// worker-build downloads at the versions it pins, from the versions in
// src/versions.rs and the targets in src/binary.rs.
//
//   node worker-build/scripts/update-checksums.mjs
//
// GitHub release assets take the sha256 GitHub records for each upload, read
// through the API (set GITHUB_TOKEN to avoid its rate limit). npm tarballs are
// downloaded, checked against the registry's sha512 `integrity`, and hashed.
// Also checks that CUR_EMSDK_COMMIT is the commit the emsdk tag points at.

import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";

const root = new URL("..", import.meta.url);
const versions = readFileSync(new URL("src/versions.rs", root), "utf8");
const binary = readFileSync(new URL("src/binary.rs", root), "utf8");

function version(name) {
  const m = versions.match(new RegExp(`static ${name}:[^=]*= (?:version!\\()?"([^"]+)"`));
  if (!m) throw new Error(`${name} not found in src/versions.rs`);
  return m[1];
}

// The target strings each `BinaryDep::target()` maps the host to.
function targets(impl) {
  const start = binary.indexOf(`impl BinaryDep for ${impl} {`);
  if (start < 0) throw new Error(`impl BinaryDep for ${impl} not found in src/binary.rs`);
  const body = binary.slice(
    binary.indexOf("fn target(", start),
    binary.indexOf("fn download_url(", start),
  );
  const found = [...body.matchAll(/=>\s*\{?\s*"([^"]+)"/g)].map((m) => m[1]);
  if (found.length === 0) throw new Error(`No targets found for ${impl}`);
  return [...new Set(found)];
}

async function get(url, accept) {
  const headers = { "user-agent": "worker-build-update-checksums" };
  if (accept) headers.accept = accept;
  if (process.env.GITHUB_TOKEN && url.startsWith("https://api.github.com/")) {
    headers.authorization = `Bearer ${process.env.GITHUB_TOKEN}`;
  }
  const res = await fetch(url, { headers });
  if (!res.ok) throw Object.assign(new Error(`${res.status} for ${url}`), { status: res.status });
  return res;
}

// sha256 of each asset of a GitHub release, by download URL.
async function releaseDigests(repo, tag) {
  const res = await get(`https://api.github.com/repos/${repo}/releases/tags/${tag}`);
  const release = await res.json();
  const digests = new Map();
  for (const asset of release.assets) {
    if (asset.digest?.startsWith("sha256:")) {
      digests.set(asset.browser_download_url, asset.digest.slice("sha256:".length));
    }
  }
  return digests;
}

const lines = [];

async function addRelease(repo, tag, urls) {
  let digests;
  try {
    digests = await releaseDigests(repo, tag);
  } catch (err) {
    if (err.status === 404) {
      console.warn(`skip ${repo}@${tag}: no such release`);
      return;
    }
    throw err;
  }
  for (const url of urls) {
    const digest = digests.get(url);
    if (!digest) throw new Error(`No sha256 digest for ${url}`);
    lines.push(`${digest}  ${url}`);
  }
}

// Binaryen (wasm-opt)
const wasmOpt = version("CUR_WASM_OPT_VERSION");
await addRelease(
  "WebAssembly/binaryen",
  `version_${wasmOpt}`,
  targets("WasmOpt").map(
    (t) =>
      `https://github.com/WebAssembly/binaryen/releases/download/version_${wasmOpt}/binaryen-version_${wasmOpt}-${t}.tar.gz`,
  ),
);

// wasm-bindgen: every 0.2.x from the minimum supported to the latest, since the
// version follows the project's Cargo.lock.
const [, , minPatch] = version("MIN_WASM_BINDGEN_LIB_VERSION").split(".").map(Number);
const [major, minor, latestPatch] = version("LATEST_WASM_BINDGEN_VERSION").split(".").map(Number);
for (let patch = minPatch; patch <= latestPatch; patch++) {
  const v = `${major}.${minor}.${patch}`;
  await addRelease(
    "wasm-bindgen/wasm-bindgen",
    v,
    targets("WasmBindgen<'_>").map(
      (t) =>
        `https://github.com/wasm-bindgen/wasm-bindgen/releases/download/${v}/wasm-bindgen-${v}-${t}.tar.gz`,
    ),
  );
}

// esbuild, from npm
const esbuild = version("CUR_ESBUILD_VERSION");
for (const t of targets("Esbuild")) {
  const url = `https://registry.npmjs.org/@esbuild/${t}/-/${t}-${esbuild}.tgz`;
  const meta = await (await get(`https://registry.npmjs.org/@esbuild/${t}/${esbuild}`)).json();
  if (meta.dist.tarball !== url) throw new Error(`Unexpected tarball ${meta.dist.tarball}`);
  const bytes = Buffer.from(await (await get(url)).arrayBuffer());
  const integrity = `sha512-${createHash("sha512").update(bytes).digest("base64")}`;
  if (integrity !== meta.dist.integrity) {
    throw new Error(`${url}: ${integrity} does not match the registry's ${meta.dist.integrity}`);
  }
  lines.push(`${createHash("sha256").update(bytes).digest("hex")}  ${url}`);
}

// emsdk is fetched by commit rather than hashed: check the pinned commit.
const emsdkTag = version("CUR_EMSCRIPTEN_VERSION");
const emsdkCommit = version("CUR_EMSDK_COMMIT");
let ref = (await (await get(`https://api.github.com/repos/emscripten-core/emsdk/git/ref/tags/${emsdkTag}`)).json()).object;
if (ref.type === "tag") ref = (await (await get(ref.url)).json()).object;
if (ref.sha !== emsdkCommit) {
  throw new Error(`emsdk tag ${emsdkTag} is commit ${ref.sha}, but CUR_EMSDK_COMMIT is ${emsdkCommit}`);
}

lines.sort((a, b) => a.slice(66).localeCompare(b.slice(66)));
writeFileSync(
  new URL("checksums.txt", root),
  "# sha256 of the archives worker-build downloads, checked before they are extracted.\n" +
    "# Generated by scripts/update-checksums.mjs; do not edit.\n" +
    lines.join("\n") +
    "\n",
);
console.log(`Wrote ${lines.length} checksums to worker-build/checksums.txt`);
