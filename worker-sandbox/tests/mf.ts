import { Miniflare, Response } from "miniflare";

export const mf = new Miniflare({
  scriptPath: "./build/worker/shim.mjs",
  compatibilityDate: "2023-05-18",
  cache: true,
  cachePersist: false,
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
});
