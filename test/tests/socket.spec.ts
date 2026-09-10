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
    const socket = await mf.dispatchConnect();
    const response = await new Promise<string>((resolve, reject) => {
      socket.once("data", (data) => {
        socket.destroy();
        resolve(data.toString());
      });
      socket.once("error", reject);
      socket.write("ping");
    });

    expect(response).toBe("ping");
  });
});
