import { describe, test, expect } from "vitest";
import { mf } from "./mf";

const productNames = ["Laptop", "Mouse", "Keyboard", "Monitor", "Headphones"];
const expectedTypedStrings = [
  "Product 1: Laptop - $999.99 (in stock: true)",
  "Product 2: Mouse - $29.99 (in stock: true)",
  "Product 3: Keyboard - $79.99 (in stock: false)",
  "Product 4: Monitor - $299.99 (in stock: true)",
  "Product 5: Headphones - $149.99 (in stock: false)",
];

describe("sql iterator durable object", () => {
  test("next() iterator returns typed results", async () => {
    const resp = await mf.dispatchFetch(
      "http://fake.host/sql-iterator/test/next",
    );
    expect(resp.status).toBe(200);

    const text = await resp.text();
    expect(text).toContain("next() iterator results:");

    // Check that we get all 5 products
    for (const s of expectedTypedStrings) {
      expect(text).toContain(s);
    }
  });

  test("raw() iterator returns column names and raw values", async () => {
    const resp = await mf.dispatchFetch(
      "http://fake.host/sql-iterator/test/raw",
    );
    expect(resp.status).toBe(200);

    const text = await resp.text();
    expect(text).toContain("raw() iterator results:");

    // Check column names are included
    expect(text).toContain("Columns: id, name, price, in_stock");

    // Check that we get raw data rows - should contain all products
    for (const name of productNames) {
      expect(text).toContain(`String("${name}")`);
    }

    // Check for different data types in raw format
    expect(text).toContain("Integer("); // For IDs
    expect(text).toContain("Float(999.99)");
    expect(text).toContain("Integer(1)"); // in_stock: true
    expect(text).toContain("Integer(0)"); // in_stock: false
  });

  test("different object instances have independent data", async () => {
    // Test first instance
    const resp1 = await mf.dispatchFetch(
      "http://fake.host/sql-iterator/instance1/next",
    );
    expect(resp1.status).toBe(200);
    const text1 = await resp1.text();
    expect(text1).toContain("Product 1: Laptop");

    // Test second instance - should have same seeded data
    const resp2 = await mf.dispatchFetch(
      "http://fake.host/sql-iterator/instance2/next",
    );
    expect(resp2.status).toBe(200);
    const text2 = await resp2.text();
    expect(text2).toContain("Product 1: Laptop");

    // Both should be identical since they use the same seed data
    expect(text1).toBe(text2);
  });

  test("iterator handles empty results gracefully", async () => {
    // Create a new instance to test empty query behavior
    const resp = await mf.dispatchFetch(
      "http://fake.host/sql-iterator/empty-test/raw",
    );
    expect(resp.status).toBe(200);

    const text = await resp.text();
    // Should still show column names even with data
    expect(text).toContain("Columns: id, name, price, in_stock");
  });

  test("next() iterator handles deserialization errors", async () => {
    const resp = await mf.dispatchFetch(
      "http://fake.host/sql-iterator/test/next-invalid",
    );
    expect(resp.status).toBe(200);

    const text = await resp.text();
    expect(text).toContain("next-invalid() iterator results:");
    
    // Should have 5 error messages (one for each row that failed deserialization)
    const deserializationErrors = text.match(/Error deserializing row:/g);
    expect(deserializationErrors).toBeTruthy();
    expect(deserializationErrors!.length).toBe(5);
    
    // Check that the error messages contain information about the type mismatch
    expect(text).toContain("invalid type");
  });

  test.each([
    ["root", ""],
    ["invalid", "/invalid"],
  ])("%s endpoint returns help message", async (_, path) => {
    const resp = await mf.dispatchFetch(
      `http://fake.host/sql-iterator/test${path}`,
    );
    expect(resp.status).toBe(200);

    const text = await resp.text();
    expect(text).toBe("SQL Iterator Test - try /next, /raw, or /next-invalid endpoints");
  });

  test("data consistency between next() and raw() methods", async () => {
    // Get data from next() method
    const nextResp = await mf.dispatchFetch(
      "http://fake.host/sql-iterator/consistency-test/next",
    );
    expect(nextResp.status).toBe(200);
    const nextText = await nextResp.text();

    // Get data from raw() method
    const rawResp = await mf.dispatchFetch(
      "http://fake.host/sql-iterator/consistency-test/raw",
    );
    expect(rawResp.status).toBe(200);
    const rawText = await rawResp.text();

    // Both should contain the same 5 products
    for (const name of productNames) {
      expect(nextText).toContain(name);
      expect(rawText).toContain(name);
    }

    // Raw should show 5 data rows (plus column header)
    const rowMatches = rawText.match(/Row: \[/g);
    expect(rowMatches).toBeTruthy();
    expect(rowMatches!.length).toBe(5);
  });
});
