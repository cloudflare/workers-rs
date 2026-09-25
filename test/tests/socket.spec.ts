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
  test("inbound connection closed when handler returns", async () => {
    const socket = await mf.dispatchConnect();
    const response = await new Promise<string>((resolve, reject) => {
      const chunks: Buffer[] = [];
      const timer = setTimeout(() => {
        socket.destroy();
        reject(new Error("socket was not closed within 5s"));
      }, 5_000);
      socket.on("data", (data) => chunks.push(data));
      socket.once("end", () => {
        clearTimeout(timer);
        resolve(Buffer.concat(chunks).toString());
      });
      socket.once("error", (err) => {
        clearTimeout(timer);
        reject(err);
      });
      socket.write("ping");
    });

    expect(response).toBe("ping");
  }, 10_000);
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
