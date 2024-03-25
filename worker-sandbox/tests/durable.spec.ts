import {describe, test, expect, vi} from "vitest";
import { mf } from "./mf";
import {MessageEvent} from "miniflare";

describe("durable", () => {
  test("put-raw", async () => {
    const resp = await mf.dispatchFetch("https://fake.host/durable/put-raw");
    expect(await resp.text()).toBe("ok");
  });

  test("websocket-to-durable", async () => {
    const resp = await mf.dispatchFetch("http://fake.host/durable/websocket", {
      headers: {
        upgrade: "websocket",
      },
    });
    expect(resp.webSocket).not.toBeNull();

    const socket = resp.webSocket!;
    socket.accept();

    const handlers = {
      messageHandler: (event: MessageEvent) => {
        expect(event.data).toMatch(/^10|20|30$/);
      },
      close(event: CloseEvent) {},
    };

    const messageHandlerWrapper = vi.spyOn(handlers, "messageHandler");
    const closeHandlerWrapper = vi.spyOn(handlers, "messageHandler");
    socket.addEventListener("message", handlers.messageHandler);
    socket.addEventListener("close", handlers.close);

    socket.send("hi, can you ++?");
    await new Promise((resolve) => setTimeout(resolve, 500));
    expect(messageHandlerWrapper).toHaveBeenCalledTimes(1);

    socket.send("hi again, more ++?");
    await new Promise((resolve) => setTimeout(resolve, 500));
    expect(messageHandlerWrapper).toHaveBeenCalledTimes(2);

    socket.close();
    expect(closeHandlerWrapper).toBeCalled();
  });
});

