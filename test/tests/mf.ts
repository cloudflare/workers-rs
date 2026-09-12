import { readFileSync } from "node:fs";
import { Miniflare, Request, Response, fetch } from "miniflare";

const manifest = {
  mainModule: "build/index.js",
  modules: {
    "build/index.js": {
      type: "esm" as const,
      contents: readFileSync("./build/index.js", "utf8"),
    },
    "build/index_bg.wasm": {
      type: "wasm" as const,
      contents: new Uint8Array(readFileSync("./build/index_bg.wasm")),
    },
  },
};

async function outboundFetch(request: Request): Promise<Response> {
  const url = new URL(request.url);
  if (url.origin === "https://cloudflare.com") {
    return new Response("cloudflare!");
  }
  if (url.href === "https://miniflare.mocks/delay") {
    await new Promise<void>((resolve, reject) => {
      const timeout = setTimeout(resolve, 10_000);
      request.signal.addEventListener("abort", () => {
        clearTimeout(timeout);
        reject(request.signal.reason);
      }, { once: true });
    });
    return new Response("cloudflare!");
  }
  if (url.href === "https://jsonplaceholder.typicode.com/todos/1") {
    return Response.json({
      userId: 1,
      id: 1,
      title: "delectus aut autem",
      completed: false,
    });
  }
  return fetch(request);
}

const mf_instance = new Miniflare({
  workers: [
    {
      config: {
        name: "test",
        type: "worker",
        compatibilityDate: "2025-07-24",
        cache: { enabled: true },
        manifest,
        env: {
          EXAMPLE_SECRET: { type: "text", value: "example" },
          SOME_SECRET: { type: "text", value: "secret!" },
          SOME_VARIABLE: { type: "text", value: "some value" },
          SOME_OBJECT_VARIABLE: {
            type: "json",
            value: { foo: 42, bar: "string" },
          },
          DB: { type: "d1", id: "DB" },
          SOME_NAMESPACE: { type: "kv", id: "SOME_NAMESPACE" },
          FILE_SIZES: { type: "kv", id: "FILE_SIZES" },
          TEST: { type: "kv", id: "TEST" },
          EMPTY_BUCKET: { type: "r2", name: "EMPTY_BUCKET" },
          PUT_BUCKET: { type: "r2", name: "PUT_BUCKET" },
          SEEDED_BUCKET: { type: "r2", name: "SEEDED_BUCKET" },
          DELETE_BUCKET: { type: "r2", name: "DELETE_BUCKET" },
          my_queue: { type: "queue", name: "my_queue" },
          remote: {
            type: "fetcher",
            handler: async () => new Response("hello world"),
          },
          COUNTER: { type: "durable-object", worker: "test", exportName: "Counter" },
          PUT_RAW_TEST_OBJECT: { type: "durable-object", worker: "test", exportName: "PutRawTestObject" },
          AUTO: { type: "durable-object", worker: "test", exportName: "AutoResponseObject" },
          MY_CLASS: { type: "durable-object", worker: "test", exportName: "MyClass" },
          ECHO_CONTAINER: { type: "durable-object", worker: "test", exportName: "EchoContainer" },
          SQL_COUNTER: { type: "durable-object", worker: "test", exportName: "SqlCounter" },
          SQL_ITERATOR: { type: "durable-object", worker: "test", exportName: "SqlIterator" },
          ASSETS: { type: "assets" },
          SECRETS: { type: "secrets-store-secret", storeId: "SECRET_STORE", secretName: "secret-name" },
          MISSING_SECRET: { type: "secrets-store-secret", storeId: "SECRET_STORE_MISSING", secretName: "missing-secret" },
          HTTP_ANALYTICS: { type: "analytics-engine-dataset", name: "HTTP_ANALYTICS" },
          TEST_RATE_LIMITER: {
            type: "rate-limit",
            namespace: "1",
            simple: { limit: 10, period: 60 },
          },
          EMAIL: {
            type: "send-email",
            allowedSenderAddresses: ["allowed-sender@example.com"],
            allowedDestinationAddresses: ["allowed-recipient@example.com"],
          },
        },
        exports: {
          Counter: { type: "durable-object", storage: "legacy-kv" },
          PutRawTestObject: { type: "durable-object", storage: "legacy-kv" },
          AutoResponseObject: { type: "durable-object", storage: "legacy-kv" },
          MyClass: { type: "durable-object", storage: "legacy-kv" },
          EchoContainer: {
            type: "durable-object",
            storage: "sqlite",
            container: { imageName: "worker-dev/echocontainer:latest" },
          },
          SqlCounter: { type: "durable-object", storage: "sqlite" },
          SqlIterator: { type: "durable-object", storage: "sqlite" },
        },
        triggers: [{ type: "queue", name: "my_queue", maxBatchTimeout: 1 }],
        assets: {
          directory: "./public",
          hasUserWorker: true,
        },
      },
      dev: {
        outboundService: { type: "fetcher", handler: outboundFetch },
      },
    },
  ],
});

const secretAPI = await mf_instance.getSecretsStoreSecretAPI("SECRETS");
await secretAPI().create("secret value");

export const mf = mf_instance;
export const mfUrl = await mf.ready;
