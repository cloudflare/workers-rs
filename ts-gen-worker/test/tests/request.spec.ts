import { expect, test } from "vitest";
import { FormData } from "miniflare";

import { mf, mfUrl } from "./mf";

test("basic sync request", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}request`);
  expect(resp.status).toBe(200);
});

test("basic async request", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}async-request`);
  expect(resp.status).toBe(200);
});

test("test data", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}test-data`);
  expect(await resp.text()).toBe("data ok");
});

test("headers", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}headers`, {
    method: "POST",
    headers: {
      A: "B",
    },
  });

  expect(resp.headers.get("A")).toBe("B");
});

test("secret", async () => {
  const formData = new FormData();
  formData.append("secret", "EXAMPLE_SECRET");

  const resp = await mf.dispatchFetch(`${mfUrl}is-secret`, {
    method: "POST",
    body: formData,
  });

  expect(await resp.text()).toBe("example");
});

test("form data", async () => {
  const formData = new FormData();
  formData.append("file", new Blob(["workers-rs is cool"]), "file");

  let resp = await mf.dispatchFetch(`${mfUrl}formdata-file-size`, {
    method: "POST",
    body: formData,
  });
  expect(resp.status).toBe(200);

  const hashes = (await resp.json()) as { name: string }[];

  resp = await mf.dispatchFetch(
    `${mfUrl}formdata-file-size/${hashes[0].name}`
  );
  expect(resp.status).toBe(200);
});

test("user id", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}user/example/test`);
  expect(await resp.text()).toBe("TEST user id: example");
});

test("user", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}user/example`);
  expect(await resp.json()).toMatchObject({ id: "example" });
});

test("post account id zones", async () => {
  const resp = await mf.dispatchFetch(
    `${mfUrl}account/example/zones`,
    {
      method: "POST",
    }
  );
  expect(await resp.text()).toBe("Create new zone for Account: example");
});

test("get account id zones", async () => {
  const resp = await mf.dispatchFetch(
    `${mfUrl}account/example/zones`
  );
  expect(await resp.text()).toBe(
    "Account id: example..... You get a zone, you get a zone!"
  );
});

test("async text echo", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}async-text-echo`, {
    method: "POST",
    body: "Example text!",
  });
  expect(await resp.text()).toBe("Example text!");
});

test("fetch", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}fetch`);
  expect(resp.status).toBe(200);
});

test("fetch json", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}fetch_json`);
  expect(resp.status).toBe(200);
});

test("proxy request", async () => {
  const resp = await mf.dispatchFetch(
    `${mfUrl}proxy_request/https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Encoding/contributors.txt`
  );
  expect(resp.status).toBe(200);
});

test("durable id", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}durable/example`);
  expect(await resp.text()).toContain("[durable_object]");
});

test("some secret", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}secret`);
  expect(await resp.text()).toBe("secret!");
});

test("some var", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}var`);
  expect(await resp.text()).toBe("some value");
});

test("some object var", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}object-var`);
  expect(await resp.json()).toStrictEqual({ foo: 42, bar: "string" });
});

test("kv key value", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}kv/a/b`, {
    method: "POST",
  });

  const keys = (await resp.json()) as { keys: unknown[] };
  expect(keys.keys).toHaveLength(1);
});

test("bytes", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}bytes`);
  expect(await resp.arrayBuffer()).toStrictEqual(
    new Uint8Array([1, 2, 3, 4, 5, 6, 7]).buffer
  );
});

test("api data", async () => {
  const data = { userId: 0, title: "Hi!", completed: true };
  const resp = await mf.dispatchFetch(`${mfUrl}api-data`, {
    method: "POST",
    headers: {
      "content-type": "application/json",
    },
    body: JSON.stringify(data),
  });

  expect(await resp.json()).toMatchObject({
    ...data,
    title: [...data.title].reverse().join(""),
  });
});

test("nonsense repeat", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}nonsense-repeat`, {
    method: "POST",
  });

  expect(await resp.text()).toBe("data ok");
});

test("status code", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}status/418`);
  expect(resp.status).toBe(418);
});

test("root", async () => {
  const resp = await mf.dispatchFetch(mfUrl, {
    method: "PUT",
  });

  expect(resp.headers.get("x-testing")).toBe("123");
});

test("async", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}async`, {
    method: "PUT",
  });

  expect(resp.headers.get("x-testing")).toBe("123");
});

test("catchall", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}hello-world`, {
    method: "OPTIONS",
  });

  expect(await resp.text()).toBe("/hello-world");
});

test("redirect default", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}redirect-default`, {
    redirect: "manual",
  });
  expect(resp.headers.get("location")).toBe("https://example.com/");
});

test("redirect 307", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}redirect-307`, {
    redirect: "manual",
  });
  expect(resp.headers.get("location")).toBe("https://example.com/");
  expect(resp.status).toBe(307);
});

test("now", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}now`);
  expect(resp.status).toBe(200);
});

test("wait", async () => {
  const then = Date.now();
  const resp = await mf.dispatchFetch(`${mfUrl}wait/100`);
  expect(resp.status).toBe(200);
  expect(Date.now() - then).toBeGreaterThan(100);
});

test("custom response body", async () => {
  const then = Date.now();
  const resp = await mf.dispatchFetch(`${mfUrl}wait/100`);
  expect(resp.status).toBe(200);
  expect(Date.now() - then).toBeGreaterThan(100);
});

test("init called", async () => {
  const resp = await mf.dispatchFetch(`${mfUrl}init-called`);
  expect(await resp.text()).toBe("true");
});

test("xor", async () => {
  async function* generator() {
    for (let i = 0; i < 255; i++) {
      yield new Uint8Array([i]);
    }
  }

  const resp = await mf.dispatchFetch(`${mfUrl}xor/10`, {
    method: "POST",
    body: generator(),
    duplex: "half",
  });

  const buffer = await resp.arrayBuffer();
  const bytes = new Uint8Array(buffer);

  for (let i = 0; i < 255; i++) {
    expect(bytes[i]).toBe(i ^ 10);
  }
});
