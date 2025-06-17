import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("sqlite durable object", () => {
  test("counter increments per object", async () => {
    // First access for object "alice"
    let resp = await mf.dispatchFetch("http://fake.host/sql-counter/alice");
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe("SQL counter is now 1");

    // Second access for same object should increment
    resp = await mf.dispatchFetch("http://fake.host/sql-counter/alice");
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe("SQL counter is now 2");

    // Different object name should have its own counter starting at 1
    resp = await mf.dispatchFetch("http://fake.host/sql-counter/bob");
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe("SQL counter is now 1");
  });
});
