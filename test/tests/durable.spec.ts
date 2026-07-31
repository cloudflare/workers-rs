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

  test("block-concurrency-while", async () => {
    const first = await mf.dispatchFetch(`${mfUrl}durable/block-concurrency`);
    expect(first.status).toBe(200);
    expect(await first.text()).toBe("1");

    const second = await mf.dispatchFetch(`${mfUrl}durable/block-concurrency`);
    expect(second.status).toBe(200);
    expect(await second.text()).toBe("2");
  });

  // Errors from the infallible guard flow back as values; the incrementing counter
  // proves the object was not reset.
  test("block-concurrency-while-infallible does not reset on error", async () => {
    const first = await mf.dispatchFetch(`${mfUrl}durable/block-concurrency-infallible`);
    expect(first.status).toBe(200);
    expect(await first.text()).toBe("err:simulated transient failure:1");

    const second = await mf.dispatchFetch(`${mfUrl}durable/block-concurrency-infallible`);
    expect(second.status).toBe(200);
    expect(await second.text()).toBe("err:simulated transient failure:2");
  });

  // The fallible variant resets the object when the closure returns Err: the in-memory
  // counter persists across requests (1 -> 2), then drops back to 1 after the reset.
  test("block-concurrency-while resets the object on error", async () => {
    const count = () =>
      mf
        .dispatchFetch(`${mfUrl}durable/block-concurrency-reset-count`)
        .then((r) => r.text());

    expect(await count()).toBe("1");
    expect(await count()).toBe("2");

    const triggered = await mf.dispatchFetch(
      `${mfUrl}durable/block-concurrency-reset-trigger`,
    );
    expect(triggered.status).toBe(500);

    expect(await count()).toBe("1");
  });

  // Lazy init guarded by block_concurrency_while runs once (init_runs stays 1).
  test("lazy async initialization runs once", async () => {
    const first = await mf.dispatchFetch(`${mfUrl}durable/lazy-init`);
    expect(first.status).toBe(200);
    expect(await first.text()).toBe("limit is 100, init_runs 1");

    const second = await mf.dispatchFetch(`${mfUrl}durable/lazy-init`);
    expect(second.status).toBe(200);
    expect(await second.text()).toBe("limit is 100, init_runs 1");
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

  test("id-from-name preserves name on state.id()", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}durable/hello`);
    expect(resp.status).toBe(200);
    const text = await resp.text();
    expect(text).toBe("Hello from my-durable-object!");
  });

  // The name should be available in the constructor, before any request is
  // handled (see https://github.com/cloudflare/workerd/issues/2240).
  test("id-from-name preserves name on state.id() in constructor", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}durable/ctor-name`);
    expect(resp.status).toBe(200);
    const text = await resp.text();
    expect(text).toBe("Hello from my-durable-object!");
  });

  // unique_id() DOs should not have a name on state.id().
  test("unique-id has no name on state.id()", async () => {
    const resp = await mf.dispatchFetch(`${mfUrl}durable/hello-unique`);
    expect(resp.status).toBe(200);
    const text = await resp.text();
    expect(text).toBe("Hello from unknown!");
  });
});
