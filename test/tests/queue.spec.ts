import { describe, test, expect } from "vitest";
import * as uuid from "uuid";
import { mf, mfUrl } from "./mf";

describe("queue", () => {
  test("send message to queue", async () => {
    const resp = await mf.dispatchFetch(
      `${mfUrl}queue/send/${uuid.v4()}`,
      { method: "POST" }
    );
    expect(resp.status).toBe(200);
  });

  test("receive message from queue", async () => {
    const id = uuid.v4();
    let resp = await mf.dispatchFetch(`${mfUrl}queue/send/${id}`, {
      method: "POST",
    });
    expect(resp.status).toBe(200);

    await new Promise((resolve) => setTimeout(resolve, 1200));

    resp = await mf.dispatchFetch(`${mfUrl}queue`);
    expect(resp.status).toBe(200);

    const messages = (await resp.json()) as { id: string }[];
    const message = messages.find((msg) => msg.id === id.toString());
    expect(message).toMatchObject({ id: id.toString() });
  });

  test("batch send message to queue", async () => {
    const id_1 = uuid.v4();
    const id_2 = uuid.v4();

    let resp = await mf.dispatchFetch(`${mfUrl}queue/send_batch`, {
      method: "POST",
      body: JSON.stringify([{ id: id_1, id_string: id_1 }, { id: id_2, id_string: id_2 }])
    });

    expect(resp.status).toBe(200);

    await new Promise((resolve) => setTimeout(resolve, 1200));

    resp = await mf.dispatchFetch(`${mfUrl}queue`);
    expect(resp.status).toBe(200);

    let body = await resp.json();

    const messages = body as { id: string }[];

    const message_1 = messages.find((msg) => msg.id === id_1.toString());
    expect(message_1).toMatchObject({ id: id_1.toString() });

    const message_2 = messages.find((msg) => msg.id === id_2.toString());
    expect(message_2).toMatchObject({ id: id_2.toString() });
  });

  test("consumer error retries the batch without resetting the instance", async () => {
    const received = async () => {
      const resp = await mf.dispatchFetch(`${mfUrl}queue`);
      expect(resp.status).toBe(200);
      return ((await resp.json()) as { id: string }[]).map((msg) => msg.id);
    };

    // A message the consumer handles normally. It is kept in a static.
    const kept = uuid.v4();
    let resp = await mf.dispatchFetch(`${mfUrl}queue/send/${kept}`, {
      method: "POST",
    });
    expect(resp.status).toBe(200);
    await expect.poll(received, { timeout: 5000 }).toContain(kept);

    // A message the consumer rejects with `Err` on its first delivery.
    const retried = uuid.v4();
    resp = await mf.dispatchFetch(`${mfUrl}queue/send_batch`, {
      method: "POST",
      body: JSON.stringify([{ id: retried, id_string: `fail-once:${retried}` }]),
    });
    expect(resp.status).toBe(200);

    // The batch is retried and succeeds on the second delivery. That needs the
    // static that remembers the first failure, so the Wasm instance must not
    // have been reinitialized by the `Err`.
    await expect.poll(received, { timeout: 10000 }).toContain(retried);
    // The state from before the `Err` is still there.
    expect(await received()).toContain(kept);
  });
});
