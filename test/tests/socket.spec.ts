import {describe, test, expect} from "vitest";
import { connect } from "node:net";
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
    const response = await new Promise<Buffer>((resolve, reject) => {
      const socket = connect(25001, "127.0.0.1", () => socket.write("ping"));
      socket.once("data", (data) => {
        socket.destroy();
        resolve(data);
      });
      socket.once("error", reject);
    });

    expect(response.toString()).toBe("ping");
  });
});
