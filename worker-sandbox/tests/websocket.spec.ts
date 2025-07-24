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

    let cnt = 0;
    socket.addEventListener("message", function (_event: MessageEvent) {
      cnt++;
    });
    let calledClose = false;
    socket.addEventListener("close", function (_event: CloseEvent) {
      calledClose = true;
    });

    socket.send("Hello, world!");

    await new Promise((resolve) => setTimeout(resolve, 500));

    expect(cnt).toBe(1);
    socket.close();

    await new Promise(resolve => setTimeout(resolve, 1000));
    expect(calledClose).toBe(true);
  });
});
