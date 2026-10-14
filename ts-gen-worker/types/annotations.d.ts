// Tags layered onto `@cloudflare/workers-types` by `ts-gen --annotate`.
// Each declaration re-states only the members it tags; names must match
// upstream exactly or generation fails.

// `@throws` is limited to classes the Fetch and Streams specs pin.

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
   * @throws {RangeError} if `status` is not a redirect status
   * @throws {TypeError} if `url` is not a valid URL
   */
  static redirect(url: string, status?: number): Response;
  /** @throws {TypeError} if `any` cannot be serialized or `maybeInit` is invalid */
  static json(any: any, maybeInit?: ResponseInit | Response): Response;
  /** @throws {TypeError} if the body has already been used */
  clone(): Response;
}
