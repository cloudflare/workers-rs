import {describe, test, expect} from "vitest";
import { mf, mfExperimental, mfUrl } from "./mf-socket";

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
  test("inbound connection with experimental", async () => {
    const socket = await mfExperimental.dispatchConnect();
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
  test("inbound udp datagrams", async () => {
    const socket = await mfExperimental.dispatchConnect({ protocol: "udp" });
    const datagrams = ["one", "two", "three"];
    const replies = await new Promise<string[]>((resolve, reject) => {
      const received: string[] = [];
      socket.on("message", (message) => {
        received.push(message.toString());
        if (received.length === datagrams.length) {
          socket.close();
          resolve(received);
        }
      });
      socket.once("error", reject);
      for (const datagram of datagrams) {
        socket.send(datagram);
      }
    });

    expect(replies).toEqual(datagrams);
  });
});
