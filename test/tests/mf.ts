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

const mf_instance = new Miniflare({
  d1Persist: false,
  kvPersist: false,
  r2Persist: false,
  cachePersist: false,
  workers: [
    {
      scriptPath: "./build/index.js",
      compatibilityDate: "2025-07-24",
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
        AUTO: "AutoResponseObject",
        MY_CLASS: "MyClass",
        ECHO_CONTAINER: {
          className: "EchoContainer",
          useSQLite: true,
          container: {
            imageName: "worker-dev/echocontainer:latest",
          }
        },
        SQL_COUNTER: {
          className: "SqlCounter",
          useSQLite: true,
        },
        SQL_ITERATOR: {
          className: "SqlIterator",
          useSQLite: true,
        },
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
        routerConfig: {
          has_user_worker: true
        }
      },
      secretsStoreSecrets: {
        SECRETS: {
          store_id: "SECRET_STORE",
          secret_name: "secret-name"
        },
        MISSING_SECRET: {
          store_id: "SECRET_STORE_MISSING",
          secret_name: "missing-secret"
        }
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

// Seed the secret store with a value using the new API
const secretAPI = await mf_instance.getSecretsStoreSecretAPI("SECRETS");
await secretAPI().create("secret value");

export const mf = mf_instance;
export const mfUrl = await mf.ready;
