import { expect, test } from "vitest";
import { mf } from "./mf";

test("site text asset", async () => {
  const resp = await mf.dispatchFetch("https://fake.host/text.txt");
  expect(resp.status).toBe(200);
});

test("site icon asset", async () => {
  const resp = await mf.dispatchFetch("https://fake.host/favicon.ico");
  expect(resp.status).toBe(200);
});