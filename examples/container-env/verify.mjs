import assert from "node:assert/strict";

const url = process.argv[2];
assert.ok(url, "usage: node verify.mjs <deployed-worker-url>");
assert.ok(process.env.REGRESSION_TOKEN, "REGRESSION_TOKEN is required");
const response = await fetch(url, {
  method: "POST",
  headers: { authorization: `Bearer ${process.env.REGRESSION_TOKEN}` },
  signal: AbortSignal.timeout(120_000),
});
assert.equal(response.status, 200, await response.clone().text());
const result = await response.json();
const path = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/binding-path-marker";
const startup = {
  BINDING_ALPHA: 'startup spaces = quotes " and λ\nnext',
  BINDING_EMPTY: "",
  ["__proto__"]: "startup ordinary key",
  BINDING_EXEC: null,
  PATH: path,
};
assert.deepEqual(result.baseline.startup, startup);
assert.deepEqual(result.overridden.startup, startup);
assert.deepEqual(result.baseline.exec, {
  BINDING_ALPHA: null,
  BINDING_EMPTY: null,
  ["__proto__"]: null,
  BINDING_EXEC: null,
  PATH: path,
});
assert.deepEqual(result.overridden.exec, {
  BINDING_ALPHA: "exec override",
  BINDING_EMPTY: "",
  ["__proto__"]: "exec ordinary key",
  BINDING_EXEC: 'exec spaces = quotes " and λ\nnext',
  PATH: path,
});
assert.equal(result.overridden.cwd, "/tmp");
console.log(JSON.stringify({ observed_at: new Date().toISOString(), passed: true, ...result }, null, 2));
