import { Miniflare, Response } from "miniflare";
import { MockAgent } from "undici";

const mockAgent = new MockAgent();

mockAgent
  .get("https://cloudflare.com")
  .intercept({ path: "/" })
  .reply(200, "cloudflare!");

mockAgent
  .get("https://miniflare.mocks")
  .intercept({ path: "/delay" })
  .reply(200, "cloudflare!")
  .delay(10000);

mockAgent
  .get("https://jsonplaceholder.typicode.com")
  .intercept({ path: "/todos/1" })
  .reply(
    200,
    {
      userId: 1,
      id: 1,
      title: "delectus aut autem",
      completed: false,
    },
    {
      headers: {
        "content-type": "application/json",
      },
    }
  );

export const mf = new Miniflare({
  scriptPath: "./build/worker/shim.mjs",
  compatibilityDate: "2023-05-18",
  cache: true,
  cachePersist: false,
  d1Databases: ["DB"],
  d1Persist: false,
  kvPersist: false,
  r2Persist: false,
  modules: true,
  modulesRules: [
    { type: "CompiledWasm", include: ["**/*.wasm"], fallthrough: true },
  ],
  bindings: {
    EXAMPLE_SECRET: "example",
    SOME_SECRET: "secret!",
  },
  durableObjects: {
    COUNTER: "Counter",
    PUT_RAW_TEST_OBJECT: "PutRawTestObject",
  },
  kvNamespaces: ["SOME_NAMESPACE", "FILE_SIZES"],
  serviceBindings: {
    async remote() {
      return new Response("hello world");
    },
  },
  r2Buckets: ["EMPTY_BUCKET", "PUT_BUCKET", "SEEDED_BUCKET", "DELETE_BUCKET"],
  queueConsumers: {
    my_queue: {
      maxBatchTimeout: 1,
    },
  },
  queueProducers: ["my_queue", "my_queue"],
  fetchMock: mockAgent,
});
