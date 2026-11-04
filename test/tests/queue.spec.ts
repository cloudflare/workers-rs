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
});
