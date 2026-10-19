// Tags layered onto `@cloudflare/workers-types` by `ts-gen --annotate`.
// Each declaration re-states only the members it tags; names must match
// upstream exactly or generation fails.
//
// `@throws` is limited to classes the Fetch and Streams specs pin.
// `@throws {never}` is given to members that cannot throw: workerd's
// implementation was checked for each group below (a typed binding
// already rules out the argument-type errors JSG would raise).
//
// `@rust` gives a `number` the integer type `worker` exposes. u64/i64
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

// Upstream defaults `Cf` to `CfProperties<…>`, the union of the incoming
// and outgoing shapes, which erases; a bare `Request` is the incoming one.
type RequestInfo<CfHostMetadata = unknown, Cf = IncomingRequestCfProperties<CfHostMetadata>> =
  Request<CfHostMetadata, Cf> | string;
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
   * @rust status: u16
   * @throws {RangeError} if `status` is not a redirect status
   * @throws {TypeError} if `url` is not a valid URL
   */
  static redirect(url: string, status?: number): Response;
  /** @throws {TypeError} if `any` cannot be serialized or `maybeInit` is invalid */
  static json(any: any, maybeInit?: ResponseInit | Response): Response;
  /** @throws {TypeError} if the body has already been used */
  clone(): Response;
  /** @rust u16 */
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
  /** @rust u16 */
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
  /** @rust code: u16 */
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
  /** @rust u16 */
  readonly code: number;
}

/** {@string copy} */
declare class Blob {
  /** @rust usize */
  get size(): number;
  /** {@string both} */
  text(): Promise<string>;
}

/** {@string copy} */
interface DurableObjectStorage {
  /** @rust usize */
  delete(keys: string[], options?: DurableObjectPutOptions): Promise<number>;
  /** @rust i64 */
  getAlarm(options?: DurableObjectGetAlarmOptions): Promise<number | null>;
}

interface DurableObjectTransaction {
  /** @rust usize */
  delete(keys: string[], options?: DurableObjectPutOptions): Promise<number>;
  /** @rust i64 */
  getAlarm(options?: DurableObjectGetAlarmOptions): Promise<number | null>;
}

/** {@string copy} */
interface DurableObjectListOptions {
  /** @rust usize */
  limit?: number;
}

/** {@string copy} */
declare abstract class SqlStorageCursor {
  readonly columnNames: string[];
  /** @rust usize */
  get rowsRead(): number;
  /** @rust usize */
  get rowsWritten(): number;
  /**
   * Allocates the row iterator; the query runs when it is advanced
   * (`next` / `toArray` keep their failure mode).
   * @throws {never}
   */
  raw<U extends SqlStorageValue[]>(): IterableIterator<U>;
}

interface Container {
  /** @rust signo: i32 */
  signal(signo: number): void;
  /** @rust port: u16 */
  getTcpPort(port: number): Fetcher;
}

interface D1ExecResult {
  /** @rust u32 */
  count: number;
}

/** {@string copy} */
interface D1Meta {
  /** @rust usize */
  rows_read: number;
  /** @rust usize */
  rows_written: number;
  /** @rust usize */
  changes: number;
  /** @rust i64 */
  last_row_id: number;
}

/** {@string copy} */
interface R2MultipartUpload {
  /**
   * @rust partNumber: u16
   * @param value {@slice view} copied before the call returns, so a `&[u8]` passes as a view
   */
  uploadPart(partNumber: number, value: any, options?: R2UploadPartOptions): Promise<R2UploadedPart>;
}

/** {@string copy} */
interface R2UploadedPart {
  /** @rust u16 */
  partNumber: number;
}

/** {@string copy} */
interface R2ListOptions {
  /** @rust u32 */
  limit?: number;
}

interface QueueSendOptions {
  /** @rust u32 */
  delaySeconds?: number;
}

interface QueueSendBatchOptions {
  /** @rust u32 */
  delaySeconds?: number;
}

interface QueueRetryOptions {
  /** @rust u32 */
  delaySeconds?: number;
}

/** {@string copy} */
interface SocketAddress {
  /** @rust u16 */
  port: number;
}

/** {@string copy} */
interface Hyperdrive {
  /** @rust u16 */
  readonly port: number;
}

/** {@string copy} */
interface IncomingRequestCfPropertiesBase {
  /** @rust u32 */
  asn?: number;
}

interface IncomingRequestCfPropertiesBotManagementBase {
  /** @rust u32 */
  score: number;
  /** @rust u32 */
  detectionIds: number[];
}

/** {@string copy} */
interface RequestInitCfProperties {
  /** @rust i32 */
  cacheTtl?: number;
}

/** {@string copy} */
interface BasicImageTransformations {
  /** @rust usize */
  width?: number;
  /** @rust usize */
  height?: number;
  /** @rust u8 */
  blur?: number;
}

/** {@string copy} */
interface RequestInitCfPropertiesImageDraw {
  /** @rust usize */
  top?: number;
  /** @rust usize */
  left?: number;
  /** @rust usize */
  bottom?: number;
  /** @rust usize */
  right?: number;
}

/** {@string copy} */
declare abstract class R2Object {
  /** @rust u64 */
  readonly size: number;
}

/** {@string copy} */
interface KVNamespaceListKey<Metadata, Key extends string = string> {
  /** @rust u64 */
  expiration?: number;
}

/** {@string copy} */
interface KVNamespaceListOptions {
  /** @rust u64 */
  limit?: number;
}

interface KVNamespaceGetOptions<Type> {
  /** @rust u64 */
  cacheTtl?: number;
}

interface KVNamespacePutOptions {
  /** @rust u64 */
  expiration?: number;
  /** @rust u64 */
  expirationTtl?: number;
}

/** {@string copy} */
declare class File extends Blob {
  /** @rust u64 */
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

declare namespace TailStream {
  /** {@string copy} */ interface AlarmEventInfo {}
  /** {@string copy} */ interface Attribute {}
  /** {@string copy} */ interface Attributes {}
  /** {@string copy} */ interface ConnectEventInfo {}
  /** {@string copy} */ interface CustomEventInfo {}
  /** {@string copy} */ interface DiagnosticChannelEvent {}
  /** {@string copy} */ interface DroppedEventsDiagnostic {}
  /** {@string copy} */ interface EmailEventInfo {}
  /** {@string copy} */ interface Exception {}
  /** {@string copy} */ interface FetchEventInfo {}
  /** {@string copy} */ interface FetchResponseInfo {}
  /** {@string copy} */ interface Header {}
  /** {@string copy} */ interface HibernatableWebSocketEventInfo {}
  /** {@string copy} */ interface HibernatableWebSocketEventInfoClose {}
  /** {@string copy} */ interface HibernatableWebSocketEventInfoError {}
  /** {@string copy} */ interface HibernatableWebSocketEventInfoMessage {}
  /** {@string copy} */ interface JsRpcEventInfo {}
  /** {@string copy} */ interface Log {}
  /** {@string copy} */ interface Onset {}
  /** {@string copy} */ interface Outcome {}
  /** {@string copy} */ interface QueueEventInfo {}
  /** {@string copy} */ interface Return {}
  /** {@string copy} */ interface ScheduledEventInfo {}
  /** {@string copy} */ interface SpanClose {}
  /** {@string copy} */ interface SpanContext {}
  /** {@string copy} */ interface SpanOpen {}
  /** {@string copy} */ interface StreamDiagnostic {}
  /** {@string copy} */ interface TailStreamErrorInfo {}
  /** {@string copy} */ interface TraceEventInfo {}
  /** {@string copy} */ interface ScriptVersion {}
  /** {@string copy} */ interface TailEvent {}
  /** {@string copy} */ interface TracePreviewInfo {}
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
interface SubtleCrypto {
  /** @param plainText {@slice view} copied before the call returns, so a `&[u8]` passes as a view */
  encrypt(algorithm: any, key: CryptoKey, plainText: any): Promise<ArrayBuffer>;
  /** @param cipherText {@slice view} copied before the call returns, so a `&[u8]` passes as a view */
  decrypt(algorithm: any, key: CryptoKey, cipherText: any): Promise<ArrayBuffer>;
  /** @param data {@slice view} copied before the call returns, so a `&[u8]` passes as a view */
  sign(algorithm: any, key: CryptoKey, data: any): Promise<ArrayBuffer>;
  /**
   * @param signature {@slice view} copied before the call returns, so a `&[u8]` passes as a view
   * @param data {@slice view} copied before the call returns, so a `&[u8]` passes as a view
   */
  verify(algorithm: any, key: CryptoKey, signature: any, data: any): Promise<boolean>;
  /** @param data {@slice view} copied before the call returns, so a `&[u8]` passes as a view */
  digest(algorithm: any, data: any): Promise<ArrayBuffer>;
  /** @param keyData {@slice view} copied before the call returns, so a `&[u8]` passes as a view */
  importKey(format: string, keyData: any, algorithm: any, extractable: boolean, keyUsages: string[]): Promise<CryptoKey>;
  /** @param wrappedKey {@slice view} copied before the call returns, so a `&[u8]` passes as a view */
  unwrapKey(format: string, wrappedKey: any, unwrappingKey: CryptoKey, unwrapAlgorithm: any, unwrappedKeyAlgorithm: any, extractable: boolean, keyUsages: string[]): Promise<CryptoKey>;
  /**
   * @param a {@slice view} read before the call returns, so a `&[u8]` passes as a view
   * @param b {@slice view} read before the call returns, so a `&[u8]` passes as a view
   */
  timingSafeEqual(a: any, b: any): boolean;
}

// `{@slice handle}`: the callee needs the caller's own buffer.
interface ReadableStreamBYOBRequest {
  /** @param view {@slice handle} the buffer lent by the request's `view`, transferred back; a handle, not `&[u8]` */
  respondWithNewView(view: any): void;
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
