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
  d1Persist: false,
  kvPersist: false,
  r2Persist: false,
  cachePersist: false,
  workers: [
    {
      scriptPath: "./build/worker/shim.mjs",
      compatibilityDate: "2023-05-18",
      cache: true,
      d1Databases: ["DB"],
      modules: true,
      modulesRules: [
        { type: "CompiledWasm", include: ["**/*.wasm"], fallthrough: true },
      ],
      bindings: {
        EXAMPLE_SECRET: "example",
        SOME_SECRET: "secret!",
        SOME_VARIABLE: "some value",
        SOME_OBJECT_VARIABLE: {
          foo: 42,
          bar: "string"
        },
      },
      durableObjects: {
        COUNTER: "Counter",
        PUT_RAW_TEST_OBJECT: "PutRawTestObject",
      },
      kvNamespaces: ["SOME_NAMESPACE", "FILE_SIZES", "TEST"],
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
      assets: {
        directory: "./public",
        binding: "ASSETS",
        routingConfig: {
          has_user_worker: true,
        },
        assetConfig: {},
      },
      wrappedBindings: {
        HTTP_ANALYTICS: {
          scriptName: "mini-analytics-engine" // mock out analytics engine binding to the "mini-analytics-engine" worker
        }
      }
    },
    {
      name: "mini-analytics-engine",
      modules: true,
      script: `export default function (env) {
        return {
          writeDataPoint(data) {
            console.log(data)
          }
        }
      }`
    }]
});
