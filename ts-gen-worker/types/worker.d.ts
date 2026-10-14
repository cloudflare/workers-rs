/*! *****************************************************************************
Copyright (c) Cloudflare. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License"); you may not use
this file except in compliance with the License. You may obtain a copy of the
License at http://www.apache.org/licenses/LICENSE-2.0
THIS CODE IS PROVIDED ON AN *AS IS* BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
KIND, EITHER EXPRESS OR IMPLIED, INCLUDING WITHOUT LIMITATION ANY IMPLIED
WARRANTIES OR CONDITIONS OF TITLE, FITNESS FOR A PARTICULAR PURPOSE,
MERCHANTABLITY OR NON-INFRINGEMENT.
See the Apache Version 2.0 License for specific language governing permissions
and limitations under the License.
***************************************************************************** */
/* eslint-disable */
// noinspection JSUnusedGlobalSymbols

/**
 * The **`console`** object provides access to the debugging console.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/console)
 */
declare namespace console {
  /**
   * The **`console.log()`** static method outputs a message to the console.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/log_static)
   *
   * @throws {never}
   */
  function log(...args: any[]): void;
  /**
   * The **`console.error()`** static method outputs a message to the console at the 'error' log level.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/error_static)
   *
   * @throws {never}
   */
  function error(...args: any[]): void;
  /**
   * The **`console.warn()`** static method outputs a warning message to the console at the 'warning' log level.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/warn_static)
   *
   * @throws {never}
   */
  function warn(...args: any[]): void;
  /**
   * The **`console.info()`** static method outputs a message to the console at the 'info' log level.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/info_static)
   *
   * @throws {never}
   */
  function info(...args: any[]): void;
  /**
   * The **`console.debug()`** static method outputs a message to the console at the 'debug' log level.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/console/debug_static)
   *
   * @throws {never}
   */
  function debug(...args: any[]): void;
}

/**
 * The `ReadableStream` interface of the Streams API represents a readable stream of byte data.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream)
 */
declare class ReadableStream {
  /**
   * The **`locked`** read-only property of the ReadableStream interface returns whether or not the readable stream is locked to a reader.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream/locked)
   */
  readonly locked: boolean;
  /**
   * The **`cancel()`** method of the ReadableStream interface returns a Promise that resolves when the stream is canceled.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream/cancel)
   */
  cancel(): Promise<void>;
  /**
   * The **`getReader()`** method of the ReadableStream interface creates a reader and locks the stream to it.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStream/getReader)
   *
   * @throws {TypeError} if the stream is already locked to a reader
   */
  getReader(): ReadableStreamDefaultReader;
  pipeTo(destination: WritableStream): Promise<void>;
}

/**
 * The **`Headers`** interface of the Fetch API allows you to perform various actions on HTTP request and response headers.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers)
 */
declare class Headers {
  /**
   * The **`Headers()`** constructor creates a new Headers object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/Headers)
   *
   * @throws {TypeError} if `init` contains an invalid header name or value
   */
  constructor(init?: HeadersInit);
  /**
   * The **`append()`** method of the Headers interface appends a new value onto an existing header inside a `Headers` object, or adds the header if it does not already exist.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/append)
   *
   * @throws {TypeError} if `name` is not a valid header name or `value` is not a valid header value
   */
  append(name: string, value: string): void;
  /**
   * The **`delete()`** method of the Headers interface deletes a header from the current `Headers` object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/delete)
   *
   * @throws {TypeError} if `name` is not a valid header name
   */
  delete(name: string): void;
  /**
   * The **`get()`** method of the Headers interface returns a byte string of all the values of a header within a `Headers` object with a given name.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/get)
   *
   * @throws {TypeError} if `name` is not a valid header name
   */
  get(name: string): string | null;
  /**
   * The **`has()`** method of the Headers interface returns a boolean stating whether a `Headers` object contains a certain header.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/has)
   *
   * @throws {TypeError} if `name` is not a valid header name
   */
  has(name: string): boolean;
  /**
   * The **`set()`** method of the Headers interface sets a new value for an existing header inside a `Headers` object, or adds the header if it does not already exist.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Headers/set)
   *
   * @throws {TypeError} if `name` is not a valid header name or `value` is not a valid header value
   */
  set(name: string, value: string): void;
}

type HeadersInit = Headers | Record<string, string> | [string, string][];

/**
 * The **`Request`** interface of the Fetch API represents a resource request.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request)
 */
declare class Request {
  /**
   * The **`Request()`** constructor creates a new Request object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/Request)
   *
   * @throws {TypeError} if `input` is not a valid URL or `init` is inconsistent with it
   */
  constructor(input: RequestInfo, init?: RequestInit);
  /**
   * The **`method`** read-only property of the Request interface contains the request's method (`GET`, `POST`, etc.)
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/method)
   */
  readonly method: string;
  /**
   * The **`url`** read-only property of the Request interface contains the URL of the request.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/url)
   */
  readonly url: string;
  /**
   * The **`headers`** read-only property of the Request interface contains the Headers object associated with the request.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/headers)
   */
  readonly headers: Headers;
  /**
   * The **`body`** read-only property of the Request interface is a ReadableStream of the body contents.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/body)
   */
  readonly body: ReadableStream | null;
  /**
   * The **`clone()`** method of the Request interface creates a copy of the current `Request` object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/clone)
   *
   * @throws {TypeError} if the body has already been used
   */
  clone(): Request;
  /**
   * The **`arrayBuffer()`** method of the Request interface reads the request body and returns it as a promise that resolves with an ArrayBuffer.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/arrayBuffer)
   */
  arrayBuffer(): Promise<ArrayBuffer>;
  /**
   * The **`text()`** method of the Request interface reads the request body and returns it as a promise that resolves with a string.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/text)
   */
  text(): Promise<string>;
  /**
   * The **`bodyUsed`** read-only property of the Body interface is a boolean value that indicates whether the body has been read yet.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/bodyUsed)
   */
  readonly bodyUsed: boolean;
  /**
   * The **`bytes()`** method of the Body interface takes a Response stream and reads it to completion. It returns a promise that resolves with a Uint8Array.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/bytes)
   */
  bytes(): Promise<Uint8Array>;
  /**
   * The **`json()`** method of the Body interface takes a Response stream and reads it to completion. It returns a promise which resolves with the result of parsing the body text as JSON.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/json)
   */
  json(): Promise<any>;
  /**
   * The **`formData()`** method of the Body interface takes a Response stream and reads it to completion. It returns a promise that resolves with a FormData object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/formData)
   */
  formData(): Promise<FormData>;
  /**
   * The **`signal`** read-only property of the Request interface returns the AbortSignal associated with the request.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/signal)
   */
  readonly signal: AbortSignal;
  readonly cf?: IncomingRequestCfProperties;
}

type RequestInfo = Request | string;

interface RequestInit {
  /** A string to set request's method. */
  method?: string;
  /** A Headers object, an object literal, or an array of two-item arrays to set request's headers. */
  headers?: HeadersInit;
  /** A BodyInit object or null to set request's body. */
  body?: BodyInit | null;
  /** A string indicating how the request will interact with the browser's cache to set request's redirect. */
  redirect?: string;
  /** An AbortSignal to set request's signal. */
  signal?: AbortSignal | null;
  cf?: any;
}

type BodyInit = ReadableStream | string | ArrayBuffer | ArrayBufferView | Blob | FormData;

/**
 * The **`Response`** interface of the Fetch API represents the response to a request.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response)
 */
declare class Response {
  /**
   * The **`Response()`** constructor creates a new Response object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/Response)
   *
   * @throws {TypeError} if `init.status` is out of range or a body is given for a null-body status
   */
  constructor(body?: BodyInit | null, init?: ResponseInit);
  // Not in workers-types: ts-gen drops the `null` arm of `body?: BodyInit | null`,
  // which loses `new Response(null, init)` (101 / 204 responses need a null body
  // alongside an init). Overload kept until ts-gen covers this.
  constructor(body: null, init?: ResponseInit);
  /**
   * The **`Response.redirect()`** static method returns a `Response` resulting in a redirect to the specified URL.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/redirect_static)
   *
   * @throws {RangeError} if `status` is not a redirect status
   * @throws {TypeError} if `url` is not a valid URL
   */
  static redirect(url: string, status?: number): Response;
  /**
   * The **`Response.json()`** static method returns a `Response` that contains the provided JSON data as body, and a `Content-Type` header which is set to `application/json`.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/json_static)
   *
   * @throws {TypeError} if `data` cannot be serialized or `init` is invalid
   */
  static json(data: any, init?: ResponseInit): Response;
  /**
   * The **`status`** read-only property of the Response interface contains the HTTP status codes of the response.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/status)
   */
  readonly status: number;
  /**
   * The **`statusText`** read-only property of the Response interface contains the status message corresponding to the HTTP status code in Response.status.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/statusText)
   */
  readonly statusText: string;
  /**
   * The **`ok`** read-only property of the Response interface contains a Boolean stating whether the response was successful (status in the range 200-299) or not.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/ok)
   */
  readonly ok: boolean;
  /**
   * The **`headers`** read-only property of the Response interface contains the Headers object associated with the response.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/headers)
   */
  readonly headers: Headers;
  /**
   * The **`body`** read-only property of the Response interface is a ReadableStream of the body contents.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/body)
   */
  readonly body: ReadableStream | null;
  /**
   * The **`clone()`** method of the Response interface creates a clone of a response object, identical in every way, but stored in a different variable.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/clone)
   *
   * @throws {TypeError} if the body has already been used
   */
  clone(): Response;
  /**
   * The **`arrayBuffer()`** method of the Response interface reads the response body and returns it as a promise that resolves with an ArrayBuffer.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/arrayBuffer)
   */
  arrayBuffer(): Promise<ArrayBuffer>;
  /**
   * The **`text()`** method of the Response interface reads the response body and returns it as a promise that resolves with a string.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Response/text)
   */
  text(): Promise<string>;
  /**
   * The **`bodyUsed`** read-only property of the Body interface is a boolean value that indicates whether the body has been read yet.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/bodyUsed)
   */
  readonly bodyUsed: boolean;
  /**
   * The **`bytes()`** method of the Body interface takes a Response stream and reads it to completion. It returns a promise that resolves with a Uint8Array.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/bytes)
   */
  bytes(): Promise<Uint8Array>;
  /**
   * The **`json()`** method of the Body interface takes a Response stream and reads it to completion. It returns a promise which resolves with the result of parsing the body text as JSON.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/json)
   */
  json(): Promise<any>;
  /**
   * The **`formData()`** method of the Body interface takes a Response stream and reads it to completion. It returns a promise that resolves with a FormData object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Request/formData)
   */
  formData(): Promise<FormData>;
  readonly webSocket: WebSocket | null;
  readonly cf?: any;
}

interface ResponseInit {
  /** The status code for the response. */
  status?: number;
  /** The status message associated with the status code. */
  statusText?: string;
  /** A Headers object, an object literal, or an array of two-item arrays to set response's headers. */
  headers?: HeadersInit;
  cf?: any;
  webSocket?: WebSocket | null;
  encodeBody?: "automatic" | "manual";
}

/**
 * The ExecutionContext interface provides methods to manage the lifecycle of a Worker invocation.
 *
 * [Cloudflare Docs Reference](https://developers.cloudflare.com/workers/runtime-apis/context/)
 */
declare class ExecutionContext {
  /**
   * The **`waitUntil()`** method extends the lifetime of the event. It accepts a Promise-based task which the Workers runtime will execute before the handler terminates but without blocking the response.
   *
   * [Cloudflare Docs Reference](https://developers.cloudflare.com/workers/runtime-apis/context/#waituntil)
   */
  waitUntil(promise: Promise<any>): void;
  /**
   * The **`passThroughOnException()`** method prevents a runtime error response when the Worker script throws an unhandled exception. Instead, the request will be forwarded to the origin server as if the Worker did not exist.
   *
   * [Cloudflare Docs Reference](https://developers.cloudflare.com/workers/runtime-apis/context/#passthroughonexception)
   */
  passThroughOnException(): void;
}

/**
 * The ExportedHandler interface defines the handlers that a Worker can export to respond to incoming events.
 *
 * [Cloudflare Docs Reference](https://developers.cloudflare.com/workers/runtime-apis/handlers/)
 */
declare function fetch(input: RequestInfo, init?: RequestInit): Promise<Response>;

/* [MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/setTimeout) */
declare function setTimeout(
  callback: (...args: any[]) => void,
  msDelay?: number,
): number;
/* [MDN Reference](https://developer.mozilla.org/docs/Web/API/Window/clearTimeout) */
declare function clearTimeout(timeoutId: number | null): void;

/**
 * The Cache API allows fine grained control of reading and writing from the Cloudflare global network cache.
 *
 * [Cloudflare Docs Reference](https://developers.cloudflare.com/workers/runtime-apis/cache/)
 */
declare const caches: CacheStorage;
declare abstract class CacheStorage {
  /**
   * The **`open()`** method of the CacheStorage interface returns a Promise that resolves to the Cache object matching the cacheName.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/CacheStorage/open)
   */
  open(cacheName: string): Promise<Cache>;
  readonly default: Cache;
}
/**
 * The Cache API allows fine grained control of reading and writing from the Cloudflare global network cache.
 *
 * [Cloudflare Docs Reference](https://developers.cloudflare.com/workers/runtime-apis/cache/)
 */
declare abstract class Cache {
  /* [Cloudflare Docs Reference](https://developers.cloudflare.com/workers/runtime-apis/cache/#delete) */
  delete(
    request: RequestInfo,
    options?: CacheQueryOptions,
  ): Promise<boolean>;
  /* [Cloudflare Docs Reference](https://developers.cloudflare.com/workers/runtime-apis/cache/#match) */
  match(
    request: RequestInfo,
    options?: CacheQueryOptions,
  ): Promise<Response | undefined>;
  /* [Cloudflare Docs Reference](https://developers.cloudflare.com/workers/runtime-apis/cache/#put) */
  put(request: RequestInfo, response: Response): Promise<void>;
}
interface CacheQueryOptions {
  ignoreMethod?: boolean;
}

/**
 * The **`AbortController`** interface represents a controller object that allows you to abort one or more Web requests as and when desired.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/AbortController)
 */
declare class AbortController {
  constructor();
  /**
   * The **`signal`** read-only property of the AbortController interface returns an AbortSignal object instance, which can be used to communicate with/abort an asynchronous operation as desired.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/AbortController/signal)
   */
  get signal(): AbortSignal;
  /**
   * The **`abort()`** method of the AbortController interface aborts an asynchronous operation before it has completed.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/AbortController/abort)
   */
  abort(reason?: any): void;
}
/**
 * The **`AbortSignal`** interface represents a signal object that allows you to communicate with an asynchronous operation (such as a fetch request) and abort it if required via an AbortController object.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/AbortSignal)
 */
declare abstract class AbortSignal {
  /**
   * The **`aborted`** read-only property returns a value that indicates whether the asynchronous operations the signal is communicating with are aborted (`true`) or not (`false`).
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/AbortSignal/aborted)
   */
  get aborted(): boolean;
  /**
   * The **`reason`** read-only property returns a JavaScript value that indicates the abort reason.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/AbortSignal/reason)
   */
  get reason(): any;
  /**
   * The **`throwIfAborted()`** method throws the signal's abort reason if the signal has been aborted; otherwise it does nothing.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/AbortSignal/throwIfAborted)
   */
  throwIfAborted(): void;
}

/**
 * The **`Blob`** interface represents a blob, which is a file-like object of immutable, raw data; they can be read as text or binary data, or converted into a ReadableStream so its methods can be used for processing the data.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Blob)
 */
declare class Blob {
  /**
   * The **`size`** read-only property of the Blob interface returns the size of the Blob or File in bytes.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Blob/size)
   */
  get size(): number;
  /**
   * The **`type`** read-only property of the Blob interface returns the MIME type of the file.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Blob/type)
   */
  get type(): string;
  /**
   * The **`arrayBuffer()`** method of the Blob interface returns a Promise that resolves with the contents of the blob as binary data contained in an ArrayBuffer.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Blob/arrayBuffer)
   */
  arrayBuffer(): Promise<ArrayBuffer>;
  /**
   * The **`bytes()`** method of the Blob interface returns a Promise that resolves with a Uint8Array containing the contents of the blob as an array of bytes.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Blob/bytes)
   */
  bytes(): Promise<Uint8Array>;
  /**
   * The **`text()`** method of the Blob interface returns a Promise that resolves with a string containing the contents of the blob, interpreted as UTF-8.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Blob/text)
   */
  text(): Promise<string>;
  /**
   * The **`stream()`** method of the Blob interface returns a ReadableStream which upon reading returns the data contained within the Blob.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Blob/stream)
   */
  stream(): ReadableStream;
}
/**
 * The **`File`** interface provides information about files and allows JavaScript in a web page to access their content.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/File)
 */
declare class File extends Blob {
  /**
   * The **`name`** read-only property of the File interface returns the name of the file represented by a File object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/File/name)
   */
  get name(): string;
  /**
   * The **`lastModified`** read-only property of the File interface provides the last modified date of the file as the number of milliseconds since the Unix epoch (January 1, 1970 at midnight).
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/File/lastModified)
   */
  get lastModified(): number;
}
/**
 * The **`FormData`** interface provides a way to construct a set of key/value pairs representing form fields and their values, which can be sent using the fetch(), XMLHttpRequest.send() or navigator.sendBeacon() methods.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/FormData)
 */
declare class FormData {
  constructor();
  /**
   * The **`append()`** method of the FormData interface appends a new value onto an existing key inside a FormData object, or adds the key if it does not already exist.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/FormData/append)
   */
  append(name: string, value: string): void;
  /**
   * The **`append()`** method of the FormData interface appends a new value onto an existing key inside a FormData object, or adds the key if it does not already exist.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/FormData/append)
   */
  append(name: string, value: Blob, filename?: string): void;
  /**
   * The **`delete()`** method of the FormData interface deletes a key and its value(s) from a FormData object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/FormData/delete)
   */
  delete(name: string): void;
  /**
   * The **`get()`** method of the FormData interface returns the first value associated with a given key from within a FormData object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/FormData/get)
   */
  get(name: string): (File | string) | null;
  /**
   * The **`getAll()`** method of the FormData interface returns all the values associated with a given key from within a FormData object.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/FormData/getAll)
   */
  getAll(name: string): (File | string)[];
  /**
   * The **`has()`** method of the FormData interface returns whether a FormData object contains a certain key.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/FormData/has)
   */
  has(name: string): boolean;
  /**
   * The **`set()`** method of the FormData interface sets a new value for an existing key inside a FormData object, or adds the key/value if it does not already exist.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/FormData/set)
   */
  set(name: string, value: string): void;
  /**
   * The **`set()`** method of the FormData interface sets a new value for an existing key inside a FormData object, or adds the key/value if it does not already exist.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/FormData/set)
   */
  set(name: string, value: Blob, filename?: string): void;
}

/**
 * The **`ReadableStreamDefaultReader`** interface of the Streams API represents a default reader that can be used to read stream data supplied from a network (such as a fetch request).
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/ReadableStreamDefaultReader)
 */
declare class ReadableStreamDefaultReader {
  constructor(stream: ReadableStream);
  get closed(): Promise<void>;
  cancel(reason?: any): Promise<void>;
  read(): Promise<ReadableStreamReadResult>;
  releaseLock(): void;
}
interface ReadableStreamReadResult {
  done: boolean;
  value?: any;
}
/**
 * The **`WritableStream`** interface of the Streams API provides a standard abstraction for writing streaming data to a destination, known as a sink.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/WritableStream)
 */
declare class WritableStream {
  get locked(): boolean;
  abort(reason?: any): Promise<void>;
  close(): Promise<void>;
  getWriter(): WritableStreamDefaultWriter;
}
/**
 * The **`WritableStreamDefaultWriter`** interface of the Streams API is the object returned by WritableStream.getWriter() and once created locks the writer to the WritableStream ensuring that no other streams can write to the underlying sink.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/WritableStreamDefaultWriter)
 */
declare class WritableStreamDefaultWriter {
  constructor(stream: WritableStream);
  get closed(): Promise<void>;
  get ready(): Promise<void>;
  get desiredSize(): number | null;
  abort(reason?: any): Promise<void>;
  close(): Promise<void>;
  write(chunk?: any): Promise<void>;
  releaseLock(): void;
}
/**
 * The **`TransformStream`** interface of the Streams API represents a concrete implementation of the pipe chain transform stream concept.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/TransformStream)
 */
declare class TransformStream {
  constructor();
  get readable(): ReadableStream;
  get writable(): WritableStream;
}
declare class FixedLengthStream extends IdentityTransformStream {
  constructor(expectedLength: number | bigint, queuingStrategy?: IdentityTransformStreamQueuingStrategy);
}
declare class IdentityTransformStream extends TransformStream {
  constructor(queuingStrategy?: IdentityTransformStreamQueuingStrategy);
}
interface IdentityTransformStreamQueuingStrategy {
  highWaterMark?: number | bigint;
}

interface IncomingRequestCfProperties {
  readonly colo: string;
  readonly country?: string;
  readonly region?: string;
  readonly city?: string;
  readonly latitude?: string;
  readonly longitude?: string;
  readonly httpProtocol: string;
}

declare abstract class Fetcher {
  fetch(input: RequestInfo, init?: RequestInit): Promise<Response>;
}

interface KVNamespace {
  get(
    key: string,
    options?: Partial<KVNamespaceGetOptions<undefined>>,
  ): Promise<string | null>;
  get(key: string, type: "text"): Promise<string | null>;
  get(key: string, type: "arrayBuffer"): Promise<ArrayBuffer | null>;
  get(key: string, type: "stream"): Promise<ReadableStream | null>;
  get(
    key: string[],
    options?: Partial<KVNamespaceGetOptions<undefined>>,
  ): Promise<Map<string, string | null>>;
  get(key: string[], type: "text"): Promise<Map<string, string | null>>;
  list(
    options?: KVNamespaceListOptions,
  ): Promise<KVNamespaceListResult>;
  put(
    key: string,
    value: string | ArrayBuffer | ArrayBufferView | ReadableStream,
    options?: KVNamespacePutOptions,
  ): Promise<void>;
  getWithMetadata(
    key: string,
    options?: Partial<KVNamespaceGetOptions<undefined>>,
  ): Promise<KVNamespaceGetWithMetadataResult>;
  getWithMetadata(
    key: string,
    type: "text",
  ): Promise<KVNamespaceGetWithMetadataResult>;
  delete(key: string): Promise<void>;
}
interface KVNamespaceListKey {
  name: string;
  expiration?: number;
  metadata?: any;
}
interface KVNamespaceListResult {
  list_complete: boolean;
  keys: KVNamespaceListKey[];
  cursor?: string;
  cacheStatus: string | null;
}
interface KVNamespaceListOptions {
  limit?: number;
  prefix?: string | null;
  cursor?: string | null;
}
interface KVNamespaceGetOptions<Type> {
  type: Type;
  cacheTtl?: number;
}
interface KVNamespacePutOptions {
  expiration?: number;
  expirationTtl?: number;
  metadata?: any | null;
}
interface KVNamespaceGetWithMetadataResult {
  value: string | null;
  metadata: any | null;
  cacheStatus: string | null;
}

interface SecretsStoreSecret {
  /**
   * Get a secret from the Secrets Store, returning a string of the secret value
   * if it exists, or throws an error if it does not exist
   */
  get(): Promise<string>;
}

interface RateLimitOptions {
  key: string;
}
interface RateLimitOutcome {
  success: boolean;
}
interface RateLimit {
  /**
   * Rate limit a request based on the provided options.
   * @see https://developers.cloudflare.com/workers/runtime-apis/bindings/rate-limit/
   * @returns A promise that resolves with the outcome of the rate limit.
   */
  limit(options: RateLimitOptions): Promise<RateLimitOutcome>;
}

interface AnalyticsEngineDataset {
  writeDataPoint(event?: AnalyticsEngineDataPoint): void;
}
interface AnalyticsEngineDataPoint {
  indexes?: ((ArrayBuffer | string) | null)[];
  doubles?: number[];
  blobs?: ((ArrayBuffer | string) | null)[];
}

interface Queue {
  send(message: any, options?: QueueSendOptions): Promise<void>;
  sendBatch(
    messages: Iterable<MessageSendRequest>,
    options?: QueueSendBatchOptions,
  ): Promise<void>;
}
type QueueContentType = "text" | "bytes" | "json" | "v8";
interface QueueSendOptions {
  contentType?: QueueContentType;
  delaySeconds?: number;
}
interface QueueSendBatchOptions {
  delaySeconds?: number;
}
interface MessageSendRequest {
  body: any;
  contentType?: QueueContentType;
  delaySeconds?: number;
}
interface QueueRetryOptions {
  delaySeconds?: number;
}
interface Message {
  readonly id: string;
  readonly timestamp: Date;
  readonly body: any;
  readonly attempts: number;
  retry(options?: QueueRetryOptions): void;
  ack(): void;
}
interface MessageBatch {
  readonly queue: string;
  readonly messages: readonly Message[];
  retryAll(options?: QueueRetryOptions): void;
  ackAll(): void;
}

interface DurableObjectId {
  toString(): string;
  equals(other: DurableObjectId): boolean;
  readonly name?: string;
  readonly jurisdiction?: string;
}
declare abstract class DurableObjectNamespace {
  newUniqueId(
    options?: DurableObjectNamespaceNewUniqueIdOptions,
  ): DurableObjectId;
  idFromName(name: string): DurableObjectId;
  idFromString(id: string): DurableObjectId;
  get(
    id: DurableObjectId,
    options?: DurableObjectNamespaceGetDurableObjectOptions,
  ): DurableObjectStub;
  getByName(
    name: string,
    options?: DurableObjectNamespaceGetDurableObjectOptions,
  ): DurableObjectStub;
}
declare abstract class DurableObjectStub extends Fetcher {
  readonly id: DurableObjectId;
  readonly name?: string;
}
type DurableObjectJurisdiction = "eu" | "fedramp" | "fedramp-high" | "us";
interface DurableObjectNamespaceNewUniqueIdOptions {
  jurisdiction?: DurableObjectJurisdiction;
}
type DurableObjectLocationHint =
  | "wnam"
  | "enam"
  | "sam"
  | "weur"
  | "eeur"
  | "apac"
  | "apac-ne"
  | "apac-se"
  | "oc"
  | "afr"
  | "me";
type DurableObjectRoutingMode = "primary-only";
interface DurableObjectNamespaceGetDurableObjectOptions {
  locationHint?: DurableObjectLocationHint;
  routingMode?: DurableObjectRoutingMode;
}
interface DurableObjectState {
  waitUntil(promise: Promise<any>): void;
  readonly id: DurableObjectId;
  readonly storage: DurableObjectStorage;
  blockConcurrencyWhile(callback: () => Promise<any>): Promise<any>;
  acceptWebSocket(ws: WebSocket, tags?: string[]): void;
  getWebSockets(tag?: string): WebSocket[];
  setWebSocketAutoResponse(maybeReqResp?: WebSocketRequestResponsePair): void;
  getWebSocketAutoResponse(): WebSocketRequestResponsePair | null;
  getWebSocketAutoResponseTimestamp(ws: WebSocket): Date | null;
  setHibernatableWebSocketEventTimeout(timeoutMs?: number): void;
  getHibernatableWebSocketEventTimeout(): number | null;
  getTags(ws: WebSocket): string[];
  abort(reason?: string): void;
}
interface DurableObjectStorage {
  get(
    key: string,
    options?: DurableObjectGetOptions,
  ): Promise<any>;
  get(
    keys: string[],
    options?: DurableObjectGetOptions,
  ): Promise<Map<string, any>>;
  list(
    options?: DurableObjectListOptions,
  ): Promise<Map<string, any>>;
  put(
    key: string,
    value: any,
    options?: DurableObjectPutOptions,
  ): Promise<void>;
  put(
    entries: Record<string, any>,
    options?: DurableObjectPutOptions,
  ): Promise<void>;
  delete(key: string, options?: DurableObjectPutOptions): Promise<boolean>;
  delete(keys: string[], options?: DurableObjectPutOptions): Promise<number>;
  deleteAll(options?: DurableObjectPutOptions): Promise<void>;
  getAlarm(options?: DurableObjectGetAlarmOptions): Promise<number | null>;
  setAlarm(
    scheduledTime: number | Date,
    options?: DurableObjectSetAlarmOptions,
  ): Promise<void>;
  deleteAlarm(options?: DurableObjectSetAlarmOptions): Promise<void>;
  sync(): Promise<void>;
  sql: SqlStorage;
}
interface DurableObjectListOptions {
  start?: string;
  startAfter?: string;
  end?: string;
  prefix?: string;
  reverse?: boolean;
  limit?: number;
  allowConcurrency?: boolean;
  noCache?: boolean;
}
interface DurableObjectGetOptions {
  allowConcurrency?: boolean;
  noCache?: boolean;
}
interface DurableObjectGetAlarmOptions {
  allowConcurrency?: boolean;
}
interface DurableObjectPutOptions {
  allowConcurrency?: boolean;
  allowUnconfirmed?: boolean;
  noCache?: boolean;
}
interface DurableObjectSetAlarmOptions {
  allowConcurrency?: boolean;
  allowUnconfirmed?: boolean;
}
declare class WebSocketRequestResponsePair {
  constructor(request: string, response: string);
  get request(): string;
  get response(): string;
}
interface SqlStorage {
  exec(
    query: string,
    ...bindings: any[]
  ): SqlStorageCursor;
  get databaseSize(): number;
}
type SqlStorageValue = ArrayBuffer | string | number | null;
declare abstract class SqlStorageCursor {
  next(): { done?: false; value: any } | { done: true; value?: never };
  toArray(): any[];
  one(): any;
  raw(): IterableIterator<SqlStorageValue[]>;
  columnNames: string[];
  get rowsRead(): number;
  get rowsWritten(): number;
  [Symbol.iterator](): IterableIterator<any>;
}

/**
 * The **`Event`** interface represents an event which takes place on an EventTarget.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Event)
 */
declare class Event {
  constructor(type: string, init?: EventInit);
  /**
   * The **`type`** read-only property of the Event interface returns a string containing the event's type.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/Event/type)
   */
  get type(): string;
}
interface EventInit {
  bubbles?: boolean;
  cancelable?: boolean;
  composed?: boolean;
}
/**
 * The **`EventTarget`** interface is implemented by objects that can receive events and may have listeners for them.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget)
 */
declare class EventTarget {
  constructor();
  /**
   * The **`addEventListener()`** method of the EventTarget interface sets up a function that will be called whenever the specified event is delivered to the target.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget/addEventListener)
   */
  addEventListener(
    type: string,
    handler: (event: Event) => void,
    options?: EventTargetAddEventListenerOptions | boolean,
  ): void;
  /**
   * The **`removeEventListener()`** method of the EventTarget interface removes an event listener previously registered with EventTarget.addEventListener() from the target.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget/removeEventListener)
   */
  removeEventListener(
    type: string,
    handler: (event: Event) => void,
    options?: EventTargetEventListenerOptions | boolean,
  ): void;
  /**
   * The **`dispatchEvent()`** method of the EventTarget sends an Event to the object, (synchronously) invoking the affected event listeners in the appropriate order.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/EventTarget/dispatchEvent)
   */
  dispatchEvent(event: Event): boolean;
}
interface EventTargetEventListenerOptions {
  capture?: boolean;
}
interface EventTargetAddEventListenerOptions {
  capture?: boolean;
  passive?: boolean;
  once?: boolean;
  signal?: AbortSignal;
}
/**
 * The **`MessageEvent`** interface represents a message received by a target object.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/MessageEvent)
 */
declare class MessageEvent extends Event {
  /**
   * The **`data`** read-only property of the MessageEvent interface represents the data sent by the message emitter.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/MessageEvent/data)
   */
  readonly data: ArrayBuffer | string;
}
/**
 * A `CloseEvent` is sent to clients using WebSockets when the connection is closed.
 *
 * [MDN Reference](https://developer.mozilla.org/docs/Web/API/CloseEvent)
 */
declare class CloseEvent extends Event {
  /**
   * The **`code`** read-only property of the CloseEvent interface returns a WebSocket connection close code indicating the reason the connection was closed.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/CloseEvent/code)
   */
  get code(): number;
  /**
   * The **`reason`** read-only property of the CloseEvent interface returns the WebSocket connection close reason the server gave for closing the connection; that is, a concise human-readable prose explanation for the closure.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/CloseEvent/reason)
   */
  get reason(): string;
  /**
   * The **`wasClean`** read-only property of the CloseEvent interface returns true if the connection closed cleanly.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/CloseEvent/wasClean)
   */
  get wasClean(): boolean;
}
declare class WebSocket extends EventTarget {
  constructor(url: string, protocols?: string[] | string);
  accept(options?: WebSocketAcceptOptions): void;
  /**
   * The **`WebSocket.send()`** method enqueues the specified data to be transmitted to the server over the WebSocket connection, increasing the value of bufferedAmount by the number of bytes needed to contain the data. If the data can't be sent (for example, because it needs to be buffered but the buffer is full), the socket is closed automatically. The browser will throw an exception if you call send() when the connection is in the CONNECTING state. If you call send() when the connection is in the CLOSING or CLOSED states, the browser will silently discard the data.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/WebSocket/send)
   */
  send(message: (ArrayBuffer | ArrayBufferView) | string): void;
  /**
   * The **`WebSocket.close()`** method closes the WebSocket connection or connection attempt, if any. If the connection is already CLOSED, this method does nothing.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/WebSocket/close)
   */
  close(code?: number, reason?: string): void;
  serializeAttachment(attachment: any): void;
  deserializeAttachment(): any | null;
  /**
   * The **`WebSocket.readyState`** read-only property returns the current state of the WebSocket connection.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/WebSocket/readyState)
   */
  readyState: number;
  /**
   * The **`WebSocket.url`** read-only property returns the absolute URL of the WebSocket as resolved by the constructor.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/WebSocket/url)
   */
  url: string | null;
  /**
   * The **`WebSocket.protocol`** read-only property returns the name of the sub-protocol the server selected; this will be one of the strings specified in the protocols parameter when creating the WebSocket object, or the empty string if no connection is established.
   *
   * [MDN Reference](https://developer.mozilla.org/docs/Web/API/WebSocket/protocol)
   */
  protocol: string | null;
}
interface WebSocketAcceptOptions {
  /**
   * When set to `true`, receiving a server-initiated WebSocket Close frame will not
   * automatically send a reciprocal Close frame, leaving the connection in a half-open
   * state. This is useful for proxying scenarios where you need to coordinate closing
   * both sides independently. Defaults to `false` when the
   * `no_web_socket_half_open_by_default` compatibility flag is enabled.
   */
  allowHalfOpen?: boolean;
}
declare class WebSocketPair {
  constructor();
  0: WebSocket;
  1: WebSocket;
}

interface Socket {
  get readable(): ReadableStream;
  get writable(): WritableStream;
  get closed(): Promise<void>;
  get opened(): Promise<SocketInfo>;
  get upgraded(): boolean;
  get secureTransport(): "on" | "off" | "starttls";
  close(): Promise<void>;
  startTls(options?: TlsOptions): Socket;
}
interface SocketOptions {
  secureTransport?: string;
  allowHalfOpen: boolean;
  highWaterMark?: number | bigint;
}
interface SocketAddress {
  hostname: string;
  port: number;
}
interface TlsOptions {
  expectedServerHostname?: string;
}
interface SocketInfo {
  remoteAddress?: string;
  localAddress?: string;
}
declare module "cloudflare:sockets" {
  function _connect(
    address: string | SocketAddress,
    options?: SocketOptions,
  ): Socket;
  export { _connect as connect };
}
