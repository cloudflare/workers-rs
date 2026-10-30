import { describe, expect, test } from "vitest";

import { mf, mfUrl } from "./mf";
import { mf as mfNode, mfUrl as mfNodeUrl } from "./mf-nodejs-compat";

const instances = [
  { name: "numeric timer ids (compat 2025-07-24)", mf, url: mfUrl },
  { name: "Node.js Timeout objects (compat 2026-08-04)", mf: mfNode, url: mfNodeUrl },
];

describe.each(instances)("Delay with $name", ({ mf, url }) => {
  test("resolves after the duration", async () => {
    const start = Date.now();
    const resp = await mf.dispatchFetch(`${url}wait/100`);
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe("Waited!\n");
    expect(Date.now() - start).toBeGreaterThanOrEqual(100);
  });

  test("dropping a pending delay does not error", async () => {
    const resp = await mf.dispatchFetch(`${url}delay-race`);
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe("short");
  });
});
