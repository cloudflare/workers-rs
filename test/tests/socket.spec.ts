import {describe, test, expect} from "vitest";
import { mf, mfUrl } from "./mf-socket";

describe("socket", () => {
  test("failed", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}socket/failed`);
    expect(resp.status).toBe(200);
  });
  test("read", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}socket/read`);
    expect(resp.status).toBe(200);
  });
  test("inbound connection", async () => {
    expect(await roundtrip("ping")).toBe("ping");
  });

  // Needs workerd with cloudflare/workerd#7518 (rejected connect handler was fatal).
  test.skip("inbound connection error does not abort the instance", async () => {
    const before = Number(await roundtrip("stat"));
    const failed = await mf.dispatchConnect();
    await new Promise<void>((resolve) => {
      failed.once("close", () => resolve());
      failed.once("error", () => {});
      failed.write("fail");
    });
    expect(Number(await roundtrip("stat"))).toBe(before + 2);
  });
});

async function roundtrip(message: string): Promise<string> {
  const socket = await mf.dispatchConnect();
  return new Promise<string>((resolve, reject) => {
    socket.once("data", (data) => {
      socket.destroy();
      resolve(data.toString());
    });
    socket.once("error", reject);
    socket.write(message);
  });
}
