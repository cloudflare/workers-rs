// Checks that a Wasm module's `name` section holds function names from each given crate, or with
// `--no-names`, that it has no `name` section.
//
// Usage: node check-wasm-names.mjs <module.wasm> <crate-name> [...]
//        node check-wasm-names.mjs --no-names <module.wasm>
import { readFileSync } from "node:fs";

const noNames = process.argv[2] === "--no-names";
const [path, ...expected] = process.argv.slice(noNames ? 3 : 2);
if (!path || noNames === expected.length > 0) {
  console.error("usage: node check-wasm-names.mjs <module.wasm> <crate-name> [...]");
  console.error("       node check-wasm-names.mjs --no-names <module.wasm>");
  process.exit(2);
}

const bytes = readFileSync(path);
let pos = 8;

function u32() {
  let result = 0;
  let shift = 0;
  for (;;) {
    const byte = bytes[pos++];
    result |= (byte & 0x7f) << shift;
    if ((byte & 0x80) === 0) return result >>> 0;
    shift += 7;
  }
}

function name() {
  const length = u32();
  const value = bytes.toString("utf8", pos, pos + length);
  pos += length;
  return value;
}

if (bytes.readUInt32BE(0) !== 0x0061736d) throw new Error(`${path} is not a Wasm module`);

const customSections = [];
const functionNames = [];
while (pos < bytes.length) {
  const id = bytes[pos++];
  const size = u32();
  const end = pos + size;
  if (id === 0) {
    const sectionName = name();
    customSections.push(sectionName);
    if (sectionName === "name") {
      while (pos < end) {
        const subsection = bytes[pos++];
        const subsectionEnd = u32() + pos;
        if (subsection === 1) {
          for (let count = u32(); count > 0; count--) {
            u32();
            functionNames.push(name());
          }
        }
        pos = subsectionEnd;
      }
    }
  }
  pos = end;
}

const failures = [];
if (customSections.includes("name") === noNames) {
  failures.push(noNames ? "unexpected `name` section" : "no `name` section");
}
// Demangled legacy symbols start with `crate::`, and v0 symbols with `crate[hash]::`.
for (const crate of expected) {
  if (!functionNames.some((n) => n.includes(`${crate}::`) || n.includes(`${crate}[`))) {
    failures.push(`no function names from crate \`${crate}\``);
  }
}

console.log(`${path}: custom sections [${customSections.join(", ")}], ${functionNames.length} function names`);
if (failures.length > 0) {
  console.error(`FAIL: ${failures.join("; ")}`);
  process.exit(1);
}
console.log("OK");
