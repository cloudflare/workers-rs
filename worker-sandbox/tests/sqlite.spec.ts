import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

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

  test("try_from_i64 handles safe values", async () => {
    // Test with a safe large value (within JavaScript safe integer range)
    const safeValue = "9007199254740991"; // 2^53 - 1 (MAX_SAFE_INTEGER)
    const resp = await mf.dispatchFetch(
      `http://fake.host/sql-counter/safe-test/set-large/${safeValue}`,
    );
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe(`Successfully stored large value: ${safeValue}`);
  });

  test("try_from_i64 rejects unsafe values", async () => {
    // Test with value exceeding JavaScript safe integer range
    const unsafeValue = "9007199254740992"; // 2^53 (exceeds MAX_SAFE_INTEGER)
    const resp = await mf.dispatchFetch(
      `http://fake.host/sql-counter/unsafe-test/set-large/${unsafeValue}`,
    );
    expect(resp.status).toBe(200);
    
    const text = await resp.text();
    expect(text).toContain("Error: Cannot store value");
    expect(text).toContain(unsafeValue);
    expect(text).toContain("JavaScript safe range is ±9007199254740991");
  });

  test("try_from_i64 handles negative safe values", async () => {
    // Test with safe negative value
    const safeNegativeValue = "-9007199254740991"; // -(2^53 - 1) (MIN_SAFE_INTEGER)
    const resp = await mf.dispatchFetch(
      `http://fake.host/sql-counter/safe-negative-test/set-large/${safeNegativeValue}`,
    );
    expect(resp.status).toBe(200);
    expect(await resp.text()).toBe(`Successfully stored large value: ${safeNegativeValue}`);
  });

  test("try_from_i64 rejects unsafe negative values", async () => {
    // Test with negative value exceeding JavaScript safe integer range
    const unsafeNegativeValue = "-9007199254740992"; // -(2^53) (exceeds MIN_SAFE_INTEGER)
    const resp = await mf.dispatchFetch(
      `http://fake.host/sql-counter/unsafe-negative-test/set-large/${unsafeNegativeValue}`,
    );
    expect(resp.status).toBe(200);
    
    const text = await resp.text();
    expect(text).toContain("Error: Cannot store value");
    expect(text).toContain(unsafeNegativeValue);
    expect(text).toContain("JavaScript safe range is ±9007199254740991");
  });
});
