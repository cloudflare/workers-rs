/** {@string copy} */
interface Env {
  readonly GREETING: string;
  readonly EXAMPLE_SECRET: string;
  readonly SOME_SECRET: string;
  readonly SOME_VARIABLE: string;
  readonly SOME_OBJECT_VARIABLE: { foo: number; bar: string };
  readonly DB: D1Database;
  readonly SOME_NAMESPACE: KVNamespace;
  readonly FILE_SIZES: KVNamespace;
  readonly TEST: KVNamespace;
  readonly EMPTY_BUCKET: R2Bucket;
  readonly PUT_BUCKET: R2Bucket;
  readonly SEEDED_BUCKET: R2Bucket;
  readonly DELETE_BUCKET: R2Bucket;
  readonly my_queue: Queue;
  readonly remote: Fetcher;
  readonly COUNTER: DurableObjectNamespace;
  readonly PUT_RAW_TEST_OBJECT: DurableObjectNamespace;
  readonly AUTO: DurableObjectNamespace;
  readonly MY_CLASS: DurableObjectNamespace;
  readonly ECHO_CONTAINER: DurableObjectNamespace;
  readonly SQL_COUNTER: DurableObjectNamespace;
  readonly SQL_ITERATOR: DurableObjectNamespace;
  readonly ASSETS: Fetcher;
  readonly SECRETS: SecretsStoreSecret;
  readonly MISSING_SECRET: SecretsStoreSecret;
  readonly HTTP_ANALYTICS: AnalyticsEngineDataset;
  readonly TEST_RATE_LIMITER: RateLimit;
  readonly EMAIL: SendEmail;
  readonly MACRO_COUNTER: DurableObjectNamespace;
  readonly EXT_KV: KVNamespace;
}
