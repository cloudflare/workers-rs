import { describe, expect, test, vi } from "vitest";
import { MessageEvent } from "miniflare";
import { mf } from "./mf";

describe("websocket", () => {
  test("to echo", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/websocket", {
      headers: {
        upgrade: "websocket",
      },
    });
    expect(resp.webSocket).not.toBeNull();

    const socket = resp.webSocket!;
    socket.accept();

    const handlers = {
      messageHandler: (event: MessageEvent) =>
        expect(event.data).toBe("Hello, world!"),
      close(event: CloseEvent) {},
    };

    const messageHandlerWrapper = vi.spyOn(handlers, "messageHandler");
    const closeHandlerWrapper = vi.spyOn(handlers, "messageHandler");
    socket.addEventListener("message", handlers.messageHandler);
    socket.addEventListener("close", handlers.close);

    socket.send("Hello, world!");

    await new Promise((resolve) => setTimeout(resolve, 500));

    expect(messageHandlerWrapper).toBeCalled();
    socket.close();
    expect(closeHandlerWrapper).toBeCalled();
  });
});
