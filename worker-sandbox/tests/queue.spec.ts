import { describe, test, expect } from "vitest";
import * as uuid from "uuid";
import { mf } from "./mf";

describe("queue", () => {
  test("send message to queue", async () => {
    const resp = await mf.dispatchFetch(
      `https://fake.host/queue/send/${uuid.v4()}`,
      { method: "POST" }
    );
    expect(resp.status).toBe(200);
  });

  test("receive message from queue", async () => {
    const id = uuid.v4();
    let resp = await mf.dispatchFetch(`https://fake.host/queue/send/${id}`, {
      method: "POST",
    });
    expect(resp.status).toBe(200);

    await new Promise((resolve) => setTimeout(resolve, 1200));

    resp = await mf.dispatchFetch("https://fake.host/queue");
    expect(resp.status).toBe(200);

    const messages = (await resp.json()) as { id: string }[];
    const message = messages.find((msg) => msg.id === id.toString());
    expect(message).toMatchObject({ id: id.toString() });
  });
});
