// Tags layered onto `@cloudflare/workers-types` by `ts-gen --annotate`.
// Each declaration re-states only the members it tags; names must match
// upstream exactly or generation fails.
//
// `@throws` is limited to classes the Fetch and Streams specs pin.
//
// `@rust` gives a `number` the integer type `worker` exposes. Where
// `worker` uses u64/i64 (R2 sizes, KV expirations, alarm times) the
// member stays f64: wasm-bindgen crosses 64-bit integers as BigInt.
// Database sizes (`D1Meta.size_after`, `SqlStorage.databaseSize`) also
// stay f64; they exceed the 32-bit `usize` `worker` truncates them to.

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
declare class Request<CfHostMetadata = unknown, Cf = IncomingRequestCfProperties<CfHostMetadata>> {
  /** @throws {TypeError} if `input` is not a valid URL or `init` is inconsistent with it */
  constructor(input: RequestInfo, init?: RequestInit);
  /** @throws {TypeError} if the body has already been used */
  clone(): Request;
}

declare class Response {
  /** @throws {TypeError} if `init.status` is out of range or a body is given for a null-body status */
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
  status: number;
}

interface ResponseInit {
  /** @rust u16 */
  status?: number;
}

interface WebSocket {
  /** @rust code: u16 */
  close(code?: number, reason?: string): void;
}

declare class CloseEvent {
  /** @rust u16 */
  readonly code: number;
}

declare class Blob {
  /** @rust usize */
  get size(): number;
}

interface DurableObjectStorage {
  /** @rust usize */
  delete(keys: string[], options?: DurableObjectPutOptions): Promise<number>;
}

interface DurableObjectTransaction {
  /** @rust usize */
  delete(keys: string[], options?: DurableObjectPutOptions): Promise<number>;
}

interface DurableObjectListOptions {
  /** @rust usize */
  limit?: number;
}

declare abstract class SqlStorageCursor {
  /** @rust usize */
  get rowsRead(): number;
  /** @rust usize */
  get rowsWritten(): number;
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

interface D1Meta {
  /** @rust usize */
  rows_read: number;
  /** @rust usize */
  rows_written: number;
  /** @rust usize */
  changes: number;
}

interface R2MultipartUpload {
  /** @rust partNumber: u16 */
  uploadPart(partNumber: number, value: any, options?: R2UploadPartOptions): Promise<R2UploadedPart>;
}

interface R2UploadedPart {
  /** @rust u16 */
  partNumber: number;
}

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

interface SocketAddress {
  /** @rust u16 */
  port: number;
}

interface Hyperdrive {
  /** @rust u16 */
  readonly port: number;
}

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

interface RequestInitCfProperties {
  /** @rust i32 */
  cacheTtl?: number;
}

interface BasicImageTransformations {
  /** @rust usize */
  width?: number;
  /** @rust usize */
  height?: number;
  /** @rust u8 */
  blur?: number;
}

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
