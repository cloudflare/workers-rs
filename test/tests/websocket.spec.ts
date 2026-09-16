import { describe, expect, test } from "vitest";
import { MessageEvent } from "miniflare";
import { mf, mfUrl } from "./mf";

describe("websocket", () => {
  test("to echo", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}websocket`, {
      headers: {
        upgrade: "websocket",
      },
    });
    expect(resp.webSocket).not.toBeNull();

    const socket = resp.webSocket!;
    socket.accept();

    const message = new Promise<MessageEvent>((resolve) => {
      socket.addEventListener("message", resolve, { once: true });
    });
    const close = new Promise<CloseEvent>((resolve) => {
      socket.addEventListener("close", resolve, { once: true });
    });

    socket.send("Hello, world!");
    expect((await message).data).toBe("Hello, world!");
    socket.close();
    await close;
  });
});
