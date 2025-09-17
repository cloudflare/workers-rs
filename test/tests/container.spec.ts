import { describe, test, expect } from "vitest";
import { mf, mfUrl } from "./mf";
import { MessageEvent } from "miniflare";

describe("container", () => {
  let testContainer = true;
  if (!process.env.TEST_CONTAINER_NAME) {
    console.log('No container specified, skipping container test');
    testContainer = false;
  }

  (testContainer ? test : test.skip)("post-echo", async () => {
    const test_text = "Hello container!";
    const resp = await mf.dispatchFetch(`${mfUrl}container/echo`, {
      method: "POST",
      body: test_text,
    });
    expect(await resp.text()).toBe(test_text);
  });

  (testContainer ? test : test.skip)("websocket-to-container", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}container/ws`, {
      headers: {
        upgrade: "websocket",
      },
    });
    expect(resp.webSocket).not.toBeNull();

    const socket = resp.webSocket!;
    socket.accept();

    const messages = ["123", "223", "323", "abc"];

    let idx = 0;
    socket.addEventListener("message", function (event: MessageEvent) {
      expect(event.data).toBe(messages[idx]);
      idx++;
    });

    for (const msg of messages) {
      socket.send(msg);
      await new Promise((resolve) => setTimeout(resolve, 500));
    }

    socket.close();
  });
});
