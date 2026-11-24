// Tags layered onto `@cloudflare/workers-types` by `ts-gen --annotate`.
// Each declaration re-states only the members it tags; names must match
// upstream exactly or generation fails.
//
// `@throws` is limited to classes the Fetch and Streams specs pin.
// `@throws {never}` is given to members that cannot throw: workerd's
// implementation was checked for each group below (a typed binding
// already rules out the argument-type errors JSG would raise).
//
// `@tw-abi` gives a `number` the integer type `worker` exposes. u64/i64
// on a `number` (R2 sizes, KV expirations, alarm times) lower to an
// inline wrapper over an f64 extern, exact to 2^53. Database sizes
// (`D1Meta.size_after`, `SqlStorage.databaseSize`) stay f64; they
// exceed the 32-bit `usize` `worker` truncates them to.

interface Console {
  /** @throws {never} */
  log(...data: any[]): void;
  /** @throws {never} */
  error(...data: any[]): void;
  /** @throws {never} */
  warn(...data: any[]): void;
  /** @throws {never} */
  info(...data: any[]): void;
  /** @throws {never} */
  debug(...data: any[]): void;
}

interface ReadableStream {
  /** @throws {TypeError} if the stream is already locked to a reader */
  getReader(): ReadableStreamDefaultReader;
}

/** {@string copy} */
declare class Headers {
  /** @throws {TypeError} if `init` contains an invalid header name or value */
  constructor(init?: HeadersInit);
  /** @throws {TypeError} if `name` is not a valid header name or `value` is not a valid header value */
  append(name: string, value: string): void;
  /** @throws {TypeError} if `name` is not a valid header name */
  delete(name: string): void;
  /** @throws {TypeError} if `name` is not a valid header name */
  get(name: string): string | null;
  /** @throws {TypeError} if `name` is not a valid header name */
  has(name: string): boolean;
  /** @throws {TypeError} if `name` is not a valid header name or `value` is not a valid header value */
  set(name: string, value: string): void;
  /** @throws {TypeError} if `name` is not `Set-Cookie` */
  getAll(name: string): string[];
  /** @throws {never} */
  entries(): IterableIterator<[key: string, value: string]>;
  /** @throws {never} */
  keys(): IterableIterator<string>;
  /** @throws {never} */
  values(): IterableIterator<string>;
}

// workers-types drops `readonly` on these JSG instance properties, but workerd
// defines them without a setter, so an assignment throws a `TypeError`.
/** {@string copy} */
declare class Request<CfHostMetadata = unknown, Cf = IncomingRequestCfProperties<CfHostMetadata>> {
  /** @throws {TypeError} if `input` is not a valid URL or `init` is inconsistent with it */
  constructor(input: RequestInfo, init?: RequestInit);
  /** @throws {TypeError} if the body has already been used */
  clone(): Request;
  readonly method: string;
  readonly url: string;
  readonly headers: Headers;
  readonly redirect: string;
  readonly fetcher: Fetcher | null;
  readonly signal: AbortSignal;
  readonly cf?: Cf;
  readonly integrity: string;
  readonly keepalive: boolean;
  readonly cache?: "no-store" | "no-cache";
}

/** {@string copy} */
declare class Response {
  /**
   * @param body {@slice view} copied before the call returns, so a `&[u8]` passes as a view
   * @throws {TypeError} if `init.status` is out of range or a body is given for a null-body status
   */
  constructor(body?: BodyInit | null, init?: ResponseInit);
  /**
   * @tw-abi status: u16
   * @throws {RangeError} if `status` is not a redirect status
   * @throws {TypeError} if `url` is not a valid URL
   */
  static redirect(url: string, status?: number): Response;
  /** @throws {TypeError} if `any` cannot be serialized or `maybeInit` is invalid */
  static json(any: any, maybeInit?: ResponseInit | Response): Response;
  /** @throws {TypeError} if the body has already been used */
  clone(): Response;
  /** @tw-abi u16 */
  readonly status: number;
  readonly statusText: string;
  readonly headers: Headers;
  readonly ok: boolean;
  readonly redirected: boolean;
  readonly url: string;
  readonly webSocket: WebSocket | null;
  readonly cf: any | undefined;
  readonly type: "default" | "error";
}

/** {@string copy} */
interface ResponseInit {
  /** @tw-abi u16 */
  status?: number;
}

/** {@string copy} */
interface Socket {
  /** @retype */
  readonly readable: ReadableStream<Uint8Array>;
  /** @retype */
  readonly writable: WritableStream<Uint8Array>;
}
/** {@string copy} */
interface WebSocket {
  /** @tw-abi code: u16 */
  close(code?: number, reason?: string): void;
  readonly readyState: number;
  readonly url: string | null;
  readonly protocol: string | null;
  readonly extensions: string | null;
}

declare class WebSocketPair {
  readonly 0: WebSocket;
  readonly 1: WebSocket;
}

/** {@string copy} */
declare class CloseEvent {
  /** @tw-abi u16 */
  readonly code: number;
}

/** {@string copy} */
declare class Blob {
  /** @tw-abi u32 */
  get size(): number;
  /** {@string both} */
  text(): Promise<string>;
}

/** {@string copy} */
interface DurableObjectStorage {
  /** @tw-abi u32 */
  delete(keys: string[], options?: DurableObjectPutOptions): Promise<number>;
  /** @tw-abi i64 */
  getAlarm(options?: DurableObjectGetAlarmOptions): Promise<number | null>;
}

interface DurableObjectTransaction {
  /** @tw-abi u32 */
  delete(keys: string[], options?: DurableObjectPutOptions): Promise<number>;
  /** @tw-abi i64 */
  getAlarm(options?: DurableObjectGetAlarmOptions): Promise<number | null>;
}

/** {@string copy} */
interface DurableObjectListOptions {
  /** @tw-abi u32 */
  limit?: number;
}

/** {@string copy} */
declare abstract class SqlStorageCursor {
  readonly columnNames: string[];
  /** @tw-abi u32 */
  get rowsRead(): number;
  /** @tw-abi u32 */
  get rowsWritten(): number;
  /**
   * Allocates the row iterator; the query runs when it is advanced
   * (`next` / `toArray` keep their failure mode).
   * @throws {never}
   */
  raw<U extends SqlStorageValue[]>(): IterableIterator<U>;
}

interface Container {
  /** @tw-abi signo: i32 */
  signal(signo: number): void;
  /** @tw-abi port: u16 */
  getTcpPort(port: number): Fetcher;
}

interface D1ExecResult {
  /** @tw-abi u32 */
  count: number;
}

/** {@string copy} */
interface D1Meta {
  /** @tw-abi u32 */
  rows_read: number;
  /** @tw-abi u32 */
  rows_written: number;
  /** @tw-abi u32 */
  changes: number;
  /** @tw-abi i64 */
  last_row_id: number;
}

/** {@string copy} */
interface R2MultipartUpload {
  /**
   * @tw-abi partNumber: u16
   * @param value {@slice view} copied before the call returns, so a `&[u8]` passes as a view
   */
  uploadPart(partNumber: number, value: any, options?: R2UploadPartOptions): Promise<R2UploadedPart>;
}

/** {@string copy} */
interface R2UploadedPart {
  /** @tw-abi u16 */
  partNumber: number;
}

/** {@string copy} */
interface R2ListOptions {
  /** @tw-abi u32 */
  limit?: number;
}

interface QueueSendOptions {
  /** @tw-abi u32 */
  delaySeconds?: number;
}

interface QueueSendBatchOptions {
  /** @tw-abi u32 */
  delaySeconds?: number;
}

interface QueueRetryOptions {
  /** @tw-abi u32 */
  delaySeconds?: number;
}

/** {@string copy} */
interface SocketAddress {
  /** @tw-abi u16 */
  port: number;
}

/** {@string copy} */
interface Hyperdrive {
  /** @tw-abi u16 */
  readonly port: number;
}

/** {@string copy} */
interface IncomingRequestCfPropertiesBase {
  /** @tw-abi u32 */
  asn?: number;
}

interface IncomingRequestCfPropertiesBotManagementBase {
  /** @tw-abi u32 */
  score: number;
  /** @tw-abi u32 */
  detectionIds: number[];
}

/** {@string copy} */
interface RequestInitCfProperties {
  /** @tw-abi i32 */
  cacheTtl?: number;
}

/** {@string copy} */
interface BasicImageTransformations {
  /** @tw-abi u32 */
  width?: number;
  /** @tw-abi u32 */
  height?: number;
  /** @tw-abi u8 */
  blur?: number;
}

/** {@string copy} */
interface RequestInitCfPropertiesImageDraw {
  /** @tw-abi u32 */
  top?: number;
  /** @tw-abi u32 */
  left?: number;
  /** @tw-abi u32 */
  bottom?: number;
  /** @tw-abi u32 */
  right?: number;
}

/** {@string copy} */
declare abstract class R2Object {
  /** @tw-abi u64 */
  readonly size: number;
}

/** {@string copy} */
interface KVNamespaceListKey<Metadata, Key extends string = string> {
  /** @tw-abi u64 */
  expiration?: number;
}

/** {@string copy} */
interface KVNamespaceListOptions {
  /** @tw-abi u64 */
  limit?: number;
}

interface KVNamespaceGetOptions<Type> {
  /** @tw-abi u64 */
  cacheTtl?: number;
}

interface KVNamespacePutOptions {
  /** @tw-abi u64 */
  expiration?: number;
  /** @tw-abi u64 */
  expirationTtl?: number;
}

/** {@string copy} */
declare class File extends Blob {
  /** @tw-abi u64 */
  get lastModified(): number;
}

// Pure constructors: workerd's `d1-api.ts` `prepare` / `withSession` build a
// statement or session object without talking to the database;
// `getBookmark` reads a field.
declare abstract class D1Database {
  /** @throws {never} */
  prepare(query: string): D1PreparedStatement;
  /** @throws {never} */
  withSession(constraintOrBookmark?: D1SessionBookmarkOrConstraint): D1DatabaseSession;
}
/** {@string copy} */
declare abstract class D1DatabaseSession {
  /** @throws {never} */
  prepare(query: string): D1PreparedStatement;
  /** @throws {never} */
  getBookmark(): D1SessionBookmark | null;
}

// `waitUntil` records the promise and `passThroughOnException` sets a flag.
interface ExecutionContext<Props = unknown> {
  /** @throws {never} */
  waitUntil(promise: Promise<any>): void;
  /** @throws {never} */
  passThroughOnException(): void;
}

// A multipart body has no name validation (unlike `Headers`), so the
// accessors and mutators cannot throw.
declare class FormData {
  /** @throws {never} */
  constructor();
  /** @throws {never} */
  append(name: string, value: string | Blob): void;
  /** @throws {never} */
  delete(name: string): void;
  /** @throws {never} */
  get(name: string): (File | string) | null;
  /** @throws {never} */
  getAll(name: string): (File | string)[];
  /** @throws {never} */
  has(name: string): boolean;
  /** @throws {never} */
  set(name: string, value: string | Blob): void;
  /** @throws {never} */
  entries(): IterableIterator<[key: string, value: File | string]>;
  /** @throws {never} */
  keys(): IterableIterator<string>;
  /** @throws {never} */
  values(): IterableIterator<File | string>;
}

// `new AbortController()` allocates a signal; `abort()` is not listed since
// workerd propagates a throwing abort listener out of `dispatchEvent`.
declare class AbortController {
  /** @throws {never} */
  constructor();
}

// Identity accessors over an already-validated id.
/** {@string copy} */
interface DurableObjectId {
  /** @throws {never} */
  toString(): string;
  /** @throws {never} */
  equals(other: DurableObjectId): boolean;
}

// `{@string copy}` makes a string return a `String` only, without the `JsString`
// `_js_string` twin. It is the default for every declaration whose strings are
// small (names, ids, keys, tags, header values, URLs, cf fields); the twin is kept
// (`{@string both}`) only for bodies read from the platform, where handing the
// `JsString` on without a UTF-8 copy matters. KV values keep both (no default).

/** {@string copy} */ interface BlobOptions {}
/** {@string copy} */ interface CachePurgeError {}
/** {@string copy} */ interface CachePurgeOptions {}
/** {@string copy} */ interface CloseEventInit {}
/** {@string copy} */ interface CloudflareAccessContext {}
/** {@string copy} */ interface CloudflareAccessIdentity {}
/** {@string copy} */ interface ContainerDirectorySnapshot {}
/** {@string copy} */ interface ContainerDirectorySnapshotOptions {}
/** {@string copy} */ interface ContainerDirectorySnapshotRestoreParams {}
/** {@string copy} */ interface ContainerExecOptions {}
/** {@string copy} */ interface ContainerSnapshot {}
/** {@string copy} */ interface ContainerSnapshotOptions {}
/** {@string copy} */ interface ContainerSnapshotRestoreParams {}
/** {@string copy} */ interface ContainerStartupOptions {}
/** {@string copy} */ interface CryptoKeyAesKeyAlgorithm {}
/** {@string copy} */ interface CryptoKeyArbitraryKeyAlgorithm {}
/** {@string copy} */ interface CryptoKeyEllipticKeyAlgorithm {}
/** {@string copy} */ interface CryptoKeyHmacKeyAlgorithm {}
/** {@string copy} */ interface CryptoKeyKeyAlgorithm {}
/** {@string copy} */ interface CryptoKeyRsaKeyAlgorithm {}
/** {@string copy} */ interface CryptoKey {}
/** {@string copy} */ interface Crypto {}
/** {@string copy} */ interface DurableObjectState {}
/** {@string copy} */ interface DurableObjectStub {}
/** {@string copy} */ interface EmailAddress {}
/** {@string copy} */ interface EmailAttachment {}
/** {@string copy} */ interface EmailMessage {}
/** {@string copy} */ interface EmailReplyMessageBuilder {}
/** {@string copy} */ interface EmailSendResult {}
/** {@string copy} */ interface ErrorEventErrorEventInit {}
/** {@string copy} */ interface ErrorEvent {}
/** {@string copy} */ interface Event {}
/** {@string copy} */ interface FileOptions {}
/** {@string copy} */ interface IncomingRequestCfPropertiesExportedAuthenticatorMetadata {}
/** {@string copy} */ interface IncomingRequestCfPropertiesGeographicInformation {}
/** {@string copy} */ interface IncomingRequestCfPropertiesTLSClientAuth {}
/** {@string copy} */ interface IncomingRequestCfPropertiesTLSClientAuthPlaceholder {}
/** {@string copy} */ interface JsonWebKey {}
/** {@string copy} */ interface KVNamespaceGetWithMetadataResult {}
/** {@string copy} */ interface KVNamespaceListResult {}
/** {@string copy} */ interface MessageBatch {}
/** {@string copy} */ interface Message {}
/** {@string copy} */ interface PerformanceEntry {}
/** {@string copy} */ interface R2Conditional {}
/** {@string copy} */ interface R2HTTPMetadata {}
/** {@string copy} */ interface R2MultipartOptions {}
/** {@string copy} */ interface R2Objects {}
/** {@string copy} */ interface R2PutOptions {}
/** {@string copy} */ interface R2StringChecksums {}
/** {@string copy} */ interface RateLimitOptions {}
/** {@string copy} */ interface ReadableStreamGetReaderOptions {}
/** {@string copy} */ interface RequestInitCfPropertiesImage {}
/** {@string copy} */ interface RequestInitCfPropertiesVaryAcceptHeader {}
/** {@string copy} */ interface RequestInitCfPropertiesVaryAcceptLanguageHeader {}
/** {@string copy} */ interface RequestInit {}
/** {@string copy} */ interface RsaOtherPrimesInfo {}
/** {@string copy} */ interface ScheduledController {}
/** {@string copy} */ interface ScriptVersion {}
/** {@string copy} */ interface SecretsStoreSecret {}
/** {@string copy} */ interface SocketInfo {}
/** {@string copy} */ interface SocketOptions {}
/** {@string copy} */ interface SubtleCryptoDeriveKeyAlgorithm {}
/** {@string copy} */ interface SubtleCryptoEncryptAlgorithm {}
/** {@string copy} */ interface SubtleCryptoGenerateKeyAlgorithm {}
/** {@string copy} */ interface SubtleCryptoHashAlgorithm {}
/** {@string copy} */ interface SubtleCryptoImportKeyAlgorithm {}
/** {@string copy} */ interface SubtleCryptoSignAlgorithm {}
/** {@string copy} */ interface SyncKvListOptions {}
/** {@string copy} */ interface TailEvent {}
/** {@string copy} */ interface TlsOptions {}
/** {@string copy} */ interface TraceDiagnosticChannelEvent {}
/** {@string copy} */ interface TraceException {}
/** {@string copy} */ interface TraceItem {}
/** {@string copy} */ interface TraceItemEmailEventInfo {}
/** {@string copy} */ interface TraceItemFetchEventInfoRequest {}
/** {@string copy} */ interface TraceItemHibernatableWebSocketEventInfoClose {}
/** {@string copy} */ interface TraceItemHibernatableWebSocketEventInfoError {}
/** {@string copy} */ interface TraceItemHibernatableWebSocketEventInfoMessage {}
/** {@string copy} */ interface TraceItemJsRpcEventInfo {}
/** {@string copy} */ interface TraceItemQueueEventInfo {}
/** {@string copy} */ interface TraceItemScheduledEventInfo {}
/** {@string copy} */ interface TraceItemTailEventInfoTailItem {}
/** {@string copy} */ interface TraceLogErrorInfo {}
/** {@string copy} */ interface TraceLog {}
/** {@string copy} */ interface TracePreviewInfo {}
/** {@string copy} */ interface Transformer {}
/** {@string copy} */ interface UnderlyingByteSource {}
/** {@string copy} */ interface UnderlyingSink {}
/** {@string copy} */ interface UnderlyingSource {}
/** {@string copy} */ interface URL {}
/** {@string copy} */ interface URLSearchParams {}
/** {@string copy} */ interface WebSocketRequestResponsePair {}
/** {@string copy} */ interface WorkerVersionMetadata {}

/** {@string copy} */
interface Body {
  /** {@string both} */
  text(): Promise<string>;
  /** @retype */
  readonly body: ReadableStream<Uint8Array> | null;
}
/** {@string copy} */
interface R2ObjectBody {
  /** {@string both} */
  text(): Promise<string>;
  /** @retype */
  readonly body: ReadableStream<Uint8Array>;
}


// A buffer-source argument copies a `&[u8]` into a fresh `Uint8Array`
// unless tagged `{@slice view}`. The tag is only for callees that copy or
// read the bytes before returning: the Fetch spec for `Response`, workerd's
// KV and R2 `put`, and WebCrypto, which works synchronously. Transferring
// callees (`ReadableByteStreamController.enqueue`) cannot take a view.
interface KVNamespace {
  /** @param value {@slice view} copied before the call returns, so a `&[u8]` passes as a view */
  put(key: string, value: any, options?: KVNamespacePutOptions): Promise<void>;
}
interface R2Bucket {
  /** @param value {@slice view} copied before the call returns, so a `&[u8]` passes as a view */
  put(key: string, value: any, options?: R2PutOptions): Promise<R2Object>;
}
// Stopgap: `{@slice view}` on `encrypt`, `decrypt`, `sign`, `verify`, `digest`,
// `importKey` and `unwrapKey` makes ts-gen skip the method (a tagged buffer
// parameter beside a second union parameter), so those copy for now.
interface SubtleCrypto {
  /**
   * @param a {@slice view} read before the call returns, so a `&[u8]` passes as a view
   * @param b {@slice view} read before the call returns, so a `&[u8]` passes as a view
   */
  timingSafeEqual(a: any, b: any): boolean;
}

// `MessageEvent.data` is a message's text, or its binary payload as an
// `ArrayBuffer` (`binaryType = "arraybuffer"`, the default before the
// `websocket_standard_binary_type` compat date) or a `Blob` (after it).
// A `MessagePort` message is any structured clone, so a trailing `any` gives the
// enum a catch-all `JsValue` arm instead of a throw.
/** {@string copy} */
declare class MessageEvent {
  /** @retype */
  readonly data: string | ArrayBuffer | Blob | any;
}
