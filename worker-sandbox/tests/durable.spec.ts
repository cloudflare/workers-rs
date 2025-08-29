import {describe, test, expect} from "vitest";
import { mf, mfUrl } from "./mf";
import {MessageEvent} from "miniflare";

describe("durable", () => {
  test("put-raw", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}durable/put-raw`);
    expect(await resp.text()).toBe("ok");
  });

  test("websocket-to-durable", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}durable/websocket`, {
      headers: {
        upgrade: "websocket",
      },
    });
    expect(resp.webSocket).not.toBeNull();

    const socket = resp.webSocket!;
    socket.accept();

    let cnt = 0;
    socket.addEventListener("message", function (event: MessageEvent) {
      cnt++;
      expect(event.data).toMatch(/^10|20|30$/);
    });
    let calledClose = false;
    socket.addEventListener("close", function (event: CloseEvent) {
      calledClose = true;
    });

    socket.send("hi, can you ++?");
    await new Promise((resolve) => setTimeout(resolve, 500));
    expect(cnt).toBe(1);

    socket.send("hi again, more ++?");
    await new Promise((resolve) => setTimeout(resolve, 500));
    expect(cnt).toBe(2);

    socket.close();

    // TODO: Investigate why this is not passing
    // await new Promise(resolve => setTimeout(resolve, 1000));
    // expect(calledClose).toBe(true);
  });

  test("get-by-name", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}durable/get-by-name`);
    expect(resp.status).toBe(200);
    const text = await resp.text();
    expect(text).toBe("Hello from my-durable-object!");
  });

  test("get-by-name-with-location-hint", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}durable/get-by-name-with-location-hint`);
    expect(resp.status).toBe(200);
    const text = await resp.text();
    expect(text).toBe("Hello from my-durable-object!");
  });
});
