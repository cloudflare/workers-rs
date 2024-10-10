import {describe, test, expect} from "vitest";
import { mf } from "./mf-socket";

describe("socket", () => {
  test("failed", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/socket/failed");
    expect(resp.status).toBe(200);
  });
  test("read", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/socket/read");
    expect(resp.status).toBe(200);
  });
});