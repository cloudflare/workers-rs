import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";

describe("signal", () => {
  test("signal::is_registered() reports correctly", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/signal/poll");
    expect(resp.status).toBe(200);

    const body = await resp.text();
    
    const [isRegistered, isNearLimit] = body.split(":");
    expect(isRegistered).toBe("true");
    expect(isNearLimit).toBe("false");
  });
});
