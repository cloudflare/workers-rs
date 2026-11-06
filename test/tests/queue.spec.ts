import { describe, test, expect, vi } from "vitest";
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

  test("queue metrics report the backlog", async () => {
    type Metrics = {
      backlog_count: number;
      backlog_bytes: number;
      oldest_message_timestamp: number | null;
    };
    const metrics = async () => {
      const resp = await mf.dispatchFetch(`${mfUrl}queue/metrics`);
      expect(resp.status).toBe(200);
      return (await resp.json()) as Metrics;
    };

    const empty = await metrics();
    expect(empty).toEqual({
      backlog_count: 0,
      backlog_bytes: 0,
      oldest_message_timestamp: null,
    });

    const resp = await mf.dispatchFetch(`${mfUrl}queue/metrics`, {
      method: "POST",
    });
    expect(resp.status).toBe(200);

    // The message joins the backlog shortly after send() resolves.
    const backlog = await vi.waitFor(async () => {
      const m = await metrics();
      expect(m.backlog_count).toBe(1);
      return m;
    });
    expect(backlog.backlog_bytes).toBeGreaterThan(0);
    expect(backlog.oldest_message_timestamp).toBeGreaterThan(0);
    expect(backlog.oldest_message_timestamp).toBeLessThanOrEqual(Date.now());
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

    // The retried message is recorded once, on its second delivery.
    resp = await mf.dispatchFetch(`${mfUrl}queue/metadata`);
    const metadata = (await resp.json()) as {
      body_id: string;
      attempts: number;
    }[];
    expect(
      metadata.filter((m) => m.body_id === retried).map((m) => m.attempts),
    ).toEqual([2]);
    expect(metadata.find((m) => m.body_id === kept)?.attempts).toBe(1);
  });

  test("read queue message metadata", async () => {
    const id = uuid.v4();

    const resp = await mf.dispatchFetch(`${mfUrl}queue/send/${id}`, {
      method: "POST",
    });
    expect(resp.status).toBe(200);

    type Metadata = {
      message_id: string;
      body_id: string;
      timestamp: string;
      attempts: number;
    };
    const message = await vi.waitFor(
      async () => {
        const resp = await mf.dispatchFetch(`${mfUrl}queue/metadata`);
        expect(resp.status).toBe(200);
        const found = ((await resp.json()) as Metadata[]).find(
          (msg) => msg.body_id === id,
        );
        expect(found).toBeDefined();
        return found!;
      },
      { timeout: 5000 },
    );

    expect(message.attempts).toBe(1);
    expect(message.message_id).not.toBe("");
    expect(Date.parse(message.timestamp)).not.toBeNaN();
  });

  // Maps must arrive as plain objects, keeping `null` entries.
  const mapBody = (id: string) => ({
    id,
    map: { k1: "v1", k2: "v2" },
    value: { a: 1, b: { c: "d" }, n: null, list: [1, "two", null] },
  });

  test("send message with map fields to queue", async () => {
    const id = uuid.v4();
    let resp = await mf.dispatchFetch(`${mfUrl}queue/map/send`, {
      method: "POST",
      body: JSON.stringify(mapBody(id)),
    });
    expect(resp.status).toBe(200);

    await new Promise((resolve) => setTimeout(resolve, 1200));

    resp = await mf.dispatchFetch(`${mfUrl}queue/map`);
    expect(resp.status).toBe(200);

    const messages = (await resp.json()) as { id: string }[];
    const message = messages.find((msg) => msg.id === id);
    expect(message).toEqual(mapBody(id));
  });

  test("batch send messages with map fields to queue", async () => {
    const id_1 = uuid.v4();
    const id_2 = uuid.v4();
    let resp = await mf.dispatchFetch(`${mfUrl}queue/map/send_batch`, {
      method: "POST",
      body: JSON.stringify([mapBody(id_1), mapBody(id_2)]),
    });
    expect(resp.status).toBe(200);

    await new Promise((resolve) => setTimeout(resolve, 1200));

    resp = await mf.dispatchFetch(`${mfUrl}queue/map`);
    expect(resp.status).toBe(200);

    const messages = (await resp.json()) as { id: string }[];
    expect(messages.find((msg) => msg.id === id_1)).toEqual(mapBody(id_1));
    expect(messages.find((msg) => msg.id === id_2)).toEqual(mapBody(id_2));
  });
});
