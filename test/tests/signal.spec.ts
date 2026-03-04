import { describe, test, expect } from "vitest";
import { mf } from "./mf";

describe("signal", () => {
  test("Signal::poll() reports is_listening as true", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/signal/poll");
    expect(resp.status).toBe(200);

    const body = await resp.text();
    const [isListening, value] = body.split(":");

    expect(isListening).toBe("true");
    // Value should be numeric and not the reserved sentinel (0xFFFF = 65535)
    expect(Number(value)).not.toBe(65535);
  });
});
