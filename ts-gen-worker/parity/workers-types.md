# Input from `@cloudflare/workers-types`

Goal: `ts-gen-worker/src/bindings.rs` is generated from
`node_modules/@cloudflare/workers-types/index.d.ts` (4.20250923.0, 9566 lines) rather than the
hand-trimmed `types/worker.d.ts`. Done: the inventory below generates and compiles (19.8k lines,
39 generator warnings, all erasures outside the load-bearing surface or listed under Gaps), and
the ported test suite with the `ext/` layer and macros (`ts-gen-worker/test/`, 23 spec files, 176
tests, also under `chomp test:ts-gen-worker-http`) is green against it. `types/worker.d.ts` is
gone; `types/annotations.d.ts` is the only hand-written declaration file.

## Inventory: JS surface `worker` depends on

From `worker-sys/src/**` and `worker/src/**`. `web_sys`-sourced types are listed because
`ts-gen-worker` has no `web_sys`; they must come from workers-types too. Each line is an
`--export` root in `chompfile.toml` or reached through one.

- Fetch: `Request`, `RequestInit`, `RequestInfo`, `Response`, `ResponseInit`, `Headers`,
  `HeadersInit`, `BodyInit`, `fetch`, `RequestRedirect`/`RequestCache` (string unions)
- Streams: `ReadableStream`, `ReadableStreamDefaultReader`, `ReadableStreamBYOBReader`,
  `WritableStream`, `WritableStreamDefaultWriter`, `TransformStream`, `FixedLengthStream`,
  `IdentityTransformStream`, `ReadableStreamReadResult`
- Body types: `FormData`, `File`, `Blob`, `URLSearchParams`, `URL`
- Abort: `AbortController`, `AbortSignal`
- Cache: `caches` (`CacheStorage` with `default`), `Cache`, `CacheQueryOptions`
- WebSocket: `WebSocket`, `WebSocketPair`, `WebSocketRequestResponsePair`, `MessageEvent`,
  `CloseEvent`, `ErrorEvent`, `Event`, `EventTarget`
- Events: `ScheduledController`, `ExecutionContext`, `MessageBatch`, `Message`, `QueueRetryOptions`,
  `EmailEvent`/`ForwardableEmailMessage`/`EmailMessage` (`cloudflare:email`), `ExtendableEvent`
- Durable Objects: `DurableObjectNamespace`, `DurableObjectId`, `DurableObjectStub`,
  `DurableObjectState`, `DurableObjectStorage`, `DurableObjectTransaction`, `SqlStorage`,
  `SqlStorageCursor`, `DurableObjectNamespaceNewUniqueIdOptions`, `DurableObjectNamespaceGetDurableObjectOptions`,
  `DurableObjectGetOptions`/`PutOptions`/`ListOptions`, `DurableObjectSetAlarmOptions`,
  `WebSocketRequestResponsePair`, `Container`
- KV: `KVNamespace` (+ `KVNamespaceGetOptions`, `KVNamespacePutOptions`, `KVNamespaceListOptions`,
  `KVNamespaceListResult`, `KVNamespaceGetWithMetadataResult`)
- D1: `D1Database`, `D1DatabaseSession`, `D1PreparedStatement`, `D1Result`, `D1ExecResult`, `D1Meta`
- R2: `R2Bucket`, `R2Object`, `R2ObjectBody`, `R2Objects`, `R2HTTPMetadata`, `R2MultipartUpload`,
  `R2UploadedPart`, `R2Range`, `R2Checksums`, `R2GetOptions`, `R2PutOptions`, `R2ListOptions`,
  `R2Conditional`, `R2MultipartOptions`
- Queue producer: `Queue`, `QueueSendOptions`, `QueueSendBatchOptions`, `MessageSendRequest`
- Fetcher: `Fetcher`, `Service`, `DispatchNamespace` (`DynamicDispatcher`)
- Sockets: `connect` (`cloudflare:sockets`), `Socket`, `SocketOptions`, `SocketAddress`, `TlsOptions`,
  `SocketInfo`
- Hyperdrive, `AnalyticsEngineDataset` (+ `AnalyticsEngineDataPoint`), `RateLimit` (+ options/outcome),
  `SecretsStoreSecret`, `Ai`, `WorkerVersionMetadata`
- Cf: `IncomingRequestCfProperties` and its `*Base`/`BotManagement*`/`Geographic*`/`TlsClientAuth`
  parts, `RequestInitCfProperties`
- crypto: `crypto` global, `DigestStream` (`crypto.DigestStream`)
- console (`log`/`error`/`warn`/`info`/`debug`), `performance.now`, `setTimeout`/`clearTimeout`
- Not needed by `worker` today: Workflows, Vectorize, Images, Pipelines, Tail, Ai model catalog.

## Done

ts-gen, `agent/workers-types` (23 commits on `gbedford/workers-parity` at `b2d9e5c`; one
self-contained PR per commit with fixture + snapshot + CONVENTIONS; every commit passes fmt,
clippy and tests alone):

- `12f364a` Declaration-level `--export <Name>` roots with transitive closure.
- `2d2b547` Type-level generics: Rust defaults on every `pub type` parameter; constructors/statics
  bind only mentioned params; `extends = Head` bare; handle-identity `PartialEq`/`Eq` for generic
  types; `FromWasmAbi` bound for async bare-`T` returns; `Promise<T>` erases inside callback types.
- `087c1bf` Generic alias instantiations chase with substituted args (defaults naming earlier
  params substitute in order); unused alias params dropped.
- `fe7ded7` Construct-signature type params (`new <T>(…)`); merged var+interface class keeps the
  interface's body scope.
- `717f07c` Intersection aliases promote to interfaces (`extends` named parts + literal members)
  when every named part is a declared object type; branded primitives and `T & {…}` keep alias
  semantics; `Omit<…>` parts dropped, duplicate parents deduped; alias type params carried through.
- `3b44819` `T[]` over a type parameter keeps the `Array<T>` form (decided at lowering, so
  `type Boxed<T> = T[]` instantiated concretely still gives a slice).
- `f37d814` Dynamic unions: literal payload variants and string-enum splicing.
- `96b8810` Dictionary helpers: float literals for numeric discriminants.
- `44b67c2` Each overload's own return type and body scope for its expansions.
- `a8cf729` Constructor-shaped `declare var X: { new(…) }` (or `prototype`-only) without an
  interface promote to classes.
- `2bf3784` A union-of-literals part of an intersection alias merges into the interface; the
  declared members win over the union's narrowing.
- `b7fd6dd` A global instance interface collapses into its module-scoped constructor class
  (`EmailMessage` global + `cloudflare:email`).
- `6511865` Factory variants are not named after single-literal required fields, except a
  discriminated union's discriminant.
- `8492113` Dotted namespace references resolve (enums by value); namespace modules import the
  enclosing scope.
- `4deaf08` `@rust <type>` / `@rust <param>: <type>` JSDoc tags force a Rust scalar.
- `cc74115` `--annotate <side.d.ts>` layers `@rust` and `@throws` tags onto the input by name;
  unsupported side declarations and ambiguous names are errors; error paths unit-tested.
- `f3f093a` The unsuffixed name goes to the first overload's shortest prefix-compatible expansion
  (`R2Bucket.get(key)`, `D1PreparedStatement.first()`; `put(entries)` does not displace
  `put(key, value)`).
- `adecf5a` Type parameters constrained to a primitive erase to it (`KVNamespace<Key extends
  string>` → non-generic, `key: &str`; `keyof X` → `string`).
- `98976e4` Members of parents beyond the first are declared on the child; dictionary factories
  set every parent's fields; classification looks through parents; `extends Box<string>` keeps
  its type arguments.
- `e3de1cd` `Named & { … }` in parameter position synthesizes an interface extending the named
  parts (`R2Bucket.get(key, R2GetOptions & { onlyIf })`).
- `45f8b86` Overload groups survive var + interface merging (`ReadableStream.getReader()` had
  vanished).
- `7c24895` Enums inside generics, callbacks and arrays lower to their JS carrier (`JsString` /
  `Number`).
- `1063861` `--annotate`: a side declaration's (or alias's) type-parameter defaults replace the
  input's, so the bare `Request` reads `cf()` as `IncomingRequestCfProperties`.

Dropped on the rebase onto `b2d9e5c`: the `&Null` variant commit (`88596cd`), superseded by
`67fa212` (same `_with_null` shape, plus setters and the `Nullish::Null`-only rule); the duplicate
string-enum commit, superseded by `2d63fac`. `2d2b547` (type-level generic defaults) absorbed
`a643100`'s "an erasing default takes its type arguments with it" rule and its `generic-defaults`
fixture + integration test; the original fixture moved to `generic-type-level.d.ts`.

workers-rs, `gbedford/tgw-workers-types` (on `gbedford/ts-gen-worker` at `e7fa2be`: the
test-suite port on the `ext/` layer and macros, with the http feature):

- `5dca395` Generate from workers-types with `--export` roots, `--fallible --error-type
  crate::Error --error-enum JsException`, and `--annotate types/annotations.d.ts` carrying the
  `@throws` that `worker.d.ts` had plus the `Request`/`RequestInfo` `Cf` default; `worker.d.ts`
  deleted; `ext/` and the test worker adapted to the upstream declarations (list below).
- `5a9681e` Integer hints: `@rust` tags in `annotations.d.ts` for every `number` `worker` exposes as
  a ≤32-bit integer (list in the file); `ext/` and the test worker pass integer statuses, close
  codes, limits and part numbers.

## Decisions

- **Root selection is a third `--export` form** rather than a new flag. `--export` already means
  "output surface"; a bare dotted identifier is unambiguous against file paths and module
  specifiers in practice (a `declare module` of the same bare name wins). Closure follows member
  signatures, extends, alias targets, generic args, callbacks, `@throws`, type-param defaults; not
  constraints.
- **Every generic `pub type` parameter gets a Rust default** (`= JsValue` when TS has none).
  workers-types uses phantom generics everywhere (`Request<CfHostMetadata, Cf>`,
  `ReadableStream<R>`, `EventTarget<EventMap>`); without defaults the bare names are unusable in
  `extends =`, return types and user code. Constructors return the bare name unless their args
  mention a parameter, so `Headers::new()` / `TransformStream::new()` need no turbofish.
- **Generic types get manual `PartialEq`/`Eq`** (handle identity) instead of derives: a derived
  impl bounds `PhantomData<T>` by `T: Eq`, which `JsValue` fails, and wasm-bindgen embeds the
  parent in subclasses so `AbortSignal: EventTarget<…>` could not derive `Eq`.
- **Intersections lower as interface-with-extends**; the first parent is reached through `Deref`
  and the others' members are flattened onto the child (below).
- `T[]` over a type parameter → `Array<T>`: wasm-bindgen has no slice ABI for a generic element.
- **Integer hints are a JSDoc tag (`@rust u16`), not a branded alias (`type u16 = number`).**
  Upstream never uses such an alias, so it would need layering anyway, and an alias in `.d.ts`
  suggests a TS-level distinction that does not exist.
- **`@rust u64`/`i64` apply to `bigint` only.** wasm-bindgen crosses 64-bit integers as `BigInt`
  (`BigInt.asUintN(64, x)` on return, a `BigInt` assertion on arguments), so `u64` on a JS number
  throws at runtime. The tag is refused with an error diagnostic. Consequence: R2 sizes, KV
  expirations/TTLs, alarm times and `File.lastModified` stay `f64` where `worker` has `u64`/`i64`
  (open problem below).
- **Annotations layer from a side file (`--annotate`), matched by name.** The side file
  re-declares only tagged members; `@rust` applies as if written on the input (a property's tag
  reaches its setter and builder), `@throws` replaces the failure mode, a side member applies to
  every overload of the input method (a return `@rust` only to overloads whose return can carry
  it: `delete(key): Promise<boolean>` vs `delete(keys): Promise<number>`), and its rendered
  `## Errors` doc is appended once. Anything named in the side file that the input lacks, or a tag
  the input type cannot carry, fails the run: the file is a curated list, a typo must be loud.
- **`@throws` moved verbatim from `worker.d.ts` to `annotations.d.ts`**, with the prose following
  upstream parameter names (`Response.json(any, maybeInit)`).
- **Integer types mirror `worker`** (u16 status/close codes/ports/part numbers, u32 delays/limits/
  ASN/bot score, usize counts and image dimensions, i32 `cacheTtl`/`signo`), except database
  sizes (`D1Meta.size_after`, `SqlStorage.databaseSize`): `worker` truncates them to 32-bit
  `usize`, which overflows past 4 GiB, so they stay `f64` rather than copy the bug.
- **Factory naming regression fixed in place.** `6201e17` originally dropped the suffix of every
  single-literal field, which renamed `EmailAttachment::new_inline` to `new` and silently dropped
  the `attachment` branch (and `ReadableStreamReadResult`'s `done: true`) as name collisions; found
  because `worker/src/bindings/email.rs` drifted. The amended commit keeps a literal's suffix when
  it differs across a discriminated union's branches.
- **workers-rs branch rebased onto the errors integration** (`gbedford/ts-gen-worker`) instead of
  `f42d969`, since generation must keep the `--fallible` / `JsException` policy the test worker
  already relies on.
- **Members of parents beyond the first are flattened onto the child** (the former open problem;
  the reviewer recommended it too). The first parent stays reachable through wasm-bindgen's
  `Deref`, so only the other parents' members are re-declared; duplicate extern fns cost nothing.
  Dictionary factories include every parent's fields — `Child::new(c)` used to build an object
  missing `Base`'s required `a`, for any single-`extends` dictionary.
- **Primary overload name = first overload's shortest prefix-compatible expansion**, not pure
  shortest arity: shortest alone would hand `DurableObjectStorage.put(entries)` the bare name over
  the first-declared `put(key, value)`.
- **Primitive-constrained type parameters erase** rather than lower to `JsGeneric`: `Key extends
  string` can only be instantiated with string subtypes, all `&str` at the ABI, so a generic would
  only ever be `JsString` under another name. Bound as `Binding::Erased` in the body scope; the
  declaration drops the parameter and references drop the argument at that position.
- **`null` arm kept as a `&Null` variant** when the parameter is required or followed by later
  parameters; truncation only covers an optional trailing parameter. Setters never get one since
  `T | null` and `T | undefined` share an IR shape.
- **Enums inside generics / callbacks / arrays cross as `JsString` / `Number`**: a `Copy` enum is
  neither `JsCast` nor `JsGeneric`, so `Function<fn(Kind)>` cannot name it.
- **Intersection promotion requires every named part to be a declared object type** (review #1):
  a branded primitive or a type-parameter refinement keeps alias semantics and erases, which is
  safe where promotion produced wrong or non-compiling output.
- **Method-level type parameters that appear only in the return stay generic** (`run<T>()`,
  `list<T>()`, `get_with_metadata<Metadata>()`), requiring a turbofish in the port
  (`run::<JsValue>()`). Erasing them to their default would lose the typed-cast form that a bare
  `T` return gives (`storage.get::<JsString>(key)`). Recorded as an open problem for the
  nested-container cases.
- **`Request`'s `Cf` default is overridden from the side file** (`Cf =
  IncomingRequestCfProperties<CfHostMetadata>`, and the same on `RequestInfo` so
  `Fetcher.fetch(&Request)` accepts the bare type). Upstream's `CfProperties<…>` union erases, and
  a handler's `req.cf()` is the incoming shape; `worker` has one `cf()` too. The price: an outgoing
  `Request::new(..)` is typed with the incoming `Cf` as well, which is what `worker` does.
- **`console` stays a `thread_local_v2` static** (`console.with(|c| c.log(..))`), the honest
  lowering of `declare const console: Console`; a `console_log!` ergonomic belongs in `ext/`.

## Open problems

- **Rust prelude name collisions.** A TS type named `Result`/`Option`/`Box`/`String` would shadow
  the prelude in the generated file (none in the inventory). Proposal: extend the `js_sys`
  collision rename (`Foo_` + `pub use`) to the Rust prelude. The reviewer recommends doing it now.
- **64-bit integers carried as JS numbers.** `worker` exposes `R2Object.size`, KV
  `expiration`/`expirationTtl`/`cacheTtl`/list `limit`, `getAlarm`, `File.lastModified` as
  `u64`/`i64`; here they are `f64`. Proposal: `@rust u64` on a `number` emits the `f64` extern
  binding under a private name plus an inline wrapper converting with `as` (both directions),
  exact up to 2^53. ts-gen already emits non-extern code (factories, builders, `PartialEq`), so
  the reviewer considers the "no shims" blocker weak; needs Guy's call on the policy.
- **`--annotate` targets by bare name.** A name declared both globally and in a module is now an
  error rather than first-wins; honoring `declare module "x" { … }` / `declare namespace` in the
  side file as scopes is the fix when a case appears.
- **Uninferrable method type parameters.** `run<T = Record<…>>(): Promise<D1Result<T>>` and
  `list<T>(): Promise<Map<string, T>>` force `run::<JsValue>()` at every call. Options: keep the
  generic (current), or erase a parameter the method's arguments never mention to its default and
  let callers `unchecked_into`. The bare-`T` return (`get<T>(key): T | undefined`) is the case
  worth keeping generic; a rule keyed on "T is the whole return" vs "T is nested" is possible.
- **Overloads distinguished only by a string literal** collapse: `KVNamespace.get(key, type:
  "text" | "json" | "arrayBuffer" | "stream")` are four overloads with identical Rust parameters
  (`type: &str`), so only the first survives with `JsNullable<JsString>`; the port
  `unchecked_into`s. Proposal: name such expansions by the literal (`get_json`, `get_array_buffer`)
  with the literal baked into an inline wrapper, which is again the non-extern-shim question.

## Gaps found

- `worker`: `ResizeQuality::Specific { value: usize }` is an untagged struct variant, so serde
  writes `{"value": N}` where `RequestInitCfPropertiesImage.quality` expects a bare number.
- `worker`: `D1ResultMeta.size_after` and `SqlStorage::database_size` truncate to 32-bit `usize`.
- ts-gen-worker: `FixedLengthStream`'s `number | bigint` length, `R2Range` (union of literals),
  `D1Meta & Record<string, unknown>` and the `HeadersInit` / `RequestInfo` / `BodyInit` unions in
  `RequestInit`/`ResponseInit` setters still erase to `JsValue` (generator warnings).
- ts-gen: `Message.attempts`, `WebSocket.readyState`, `AlarmInvocationInfo.retryCount` are integers
  `worker` does not expose; untagged here (out of scope).
- ts-gen: dynamic-union enums are emitted at crate root and resolve payloads in the root scope, so
  a namespaced payload (`OnsetInfoKind(FetchEventInfo)` in TailStream) is unqualified and fails to
  compile (review #14; outside the inventory).
- ts-gen: `AttributeValueKind { VecOfBool(Vec<bool>), VecOfF64(Vec<f64>) }` payloads do not
  satisfy `ErasableGeneric` (review #18; outside the inventory). The earlier claim that `1a6f004`
  fixed `Vec<bool>`/`Vec<f64>` in dynamic unions was wrong and is corrected below.
- ts-gen: two inline option literals differing only in a literal (`raw({columnNames: true})` /
  `raw({columnNames?: false})`) synthesize `D1PreparedStatementOptions` and `…Options2`.
- ts-gen: a required nullable dictionary field (`tag: string | null`) has no way to be set to
  `null` through the factory.

## Findings on the raw input

Full-file generation (`--export index.d.ts` + every `cloudflare:*` module) started at 30,081
lines and 7,718 rustc errors. Root causes and their fix:

- Namespace `pub mod`s lacked `use super::*`: `67bb4c8`.
- String enums inside namespace modules / `&StringEnum` args: `92a5996`.
- Global `EmailMessage` vs `cloudflare:email`'s `EmailMessage` duplicate: `4fd6975`.
- Intersection aliases (`IncomingRequestCfProperties`, `D1Result`, `Fetcher`,
  `DurableObjectStub`) erased: `513b00b`, `975c771`.
- `Promise<void> | void` and literal payloads in dynamic unions: `f37d814` (`Vec<bool>`/`Vec<f64>`
  payloads are still broken, see Gaps).
- `ReadableStream.getReader()` lost by var + interface merging: `45f8b86`.
- `R2Bucket.get` / `D1PreparedStatement.first` base name on the longer overload: `f3f093a`.
- `KVNamespace<Key extends string>` generic over a string: `adecf5a`.
- `IncomingRequestCfProperties.country()` / `EmailMessageBuilder` unreachable through `AsRef`:
  `98976e4`.
- `R2GetOptions & { onlyIf }` parameters erased: `e3de1cd`.
- `new Response(null, init)`: `67fa212` (workers-rs-2; my equivalent was dropped).
- `slice_to_array` on `&[T]` over a type parameter: `b296573`.
- Still erased to `JsValue` (outside the inventory or listed above): `readonly` index signatures,
  `typeof`, `keyof`, conditional / mapped types, `this`.

Declaration-level findings (workers-types itself, not ts-gen):

- `MessageEvent.data` is `any` upstream (the hand-copied file had narrowed it to `ArrayBuffer |
  string`, which `MessageEventInit.data` does declare).
- `Request.cf` is `IncomingRequestCfProperties | RequestInitCfProperties` through the `Cf`
  default, so an incoming handler has to name `Request<unknown, IncomingRequestCfProperties>` or
  cast; the fetch handler type (`ExportedHandlerFetchHandler`) does pin it.
- `Body.body` is `ReadableStream` (`<any>`) while `BodyInit` wants `ReadableStream<Uint8Array>`,
  so piping a request body into a `Response` needs an `unchecked_into`.
- `D1Response.error?: never` lowers to `JsValue` (checked with `is_undefined()`).
- `EmailMessageBuilder` (`EmailReplyMessageBuilder & EmailDestinations`) makes `to`/`cc`/`bcc`
  individually optional with an "at least one" union the type system expresses but a factory
  cannot; `builder(from, subject).to(..)` is the resulting shape.
- `setTimeout<Args extends any[]>(callback: (...args: Args) => void, msDelay?, ...args: Args)`
  keeps `Args` generic on the binding, so a bare call needs `set_timeout::<JsValue>(…, &[])`.

## Port changes for the real input

Every mismatch between the hand-copied declarations the port (and the `ext/` layer) was written
against and what the upstream input generates, and how the code moved (all in `5dca395` unless
noted):

- `console::log(..)` → `console.with(|c| c.log(..))` (`declare const console: Console`), in
  `ext/` (`error_response`) and the test worker.
- `req.cf()` reads `IncomingRequestCfProperties` through the side-file default; `ext/interop.rs`
  passes it to `RequestInit.cf` directly.
- `Response::json_with_init` → `json_with_response_init` (upstream names the parameter
  `maybeInit: ResponseInit | Response`, so the union suffix is by type).
- `ext/streams.rs::readable_stream_from` returns `ReadableStream<Uint8Array>`, the form `BodyInit`
  names; `ext/response.rs` and `ext/interop.rs` pass `u16` statuses (integer-hints commit).
- `Response::new_with_readable_stream(&transform.readable())` → `.unchecked_into()` /
  `.unchecked_ref()` for the `ReadableStream<Uint8Array>` parameter.
- `resp.json_2()` → `resp.json::<JsValue>()` (the instance method lives on `Body`; no collision
  with the static `Response::json`).
- D1: `stmt.first(col)` / `first_2()` → `first_with_col_name(col)` / `first()`; `raw()` →
  `raw::<JsValue>()`; `run()` → `run::<JsValue>()`; `assert_eq!(all.error(), None)` →
  `assert!(all.error().is_undefined())`.
- Storage: `list()` → `list::<JsValue>()`; `get_with_keys` / `put_with_entries` take a turbofish;
  `get(key)` returns `JsOption<T>` so `storage_get` uses `into_option()`.
- KV: `get_with_metadata::<JsValue>()`, `list::<JsValue>()`; `keys().len()` → `.length()`
  (`Array<KVNamespaceListKey<_>>`); `metadata()` is `Option<T>`; bulk `get_with_slice(&[String])`
  → `get_with_array(&Array<JsString>)` (upstream spells it `Array<Key>`).
- R2: `get_with_options` → `get_with_r2_get_options`; `put_with_str_and_options` →
  `put_with_str_and_r2_put_options` (returns `R2Object`, not nullable); `get(key)` now yields
  `R2ObjectBody` so `.text()`/`.bytes()` resolve without casts.
- Streams: `reader.read()` yields `ReadableStreamReadResult<R>` with `value() -> Option<R>`.
- Email: `SendEmailBuilder::builder(from, to, subject)` → `EmailMessageBuilder::builder(from,
  subject).to(to)`; `EmailMessage::new` takes the stream (first union arm) and `new_with_str` the
  string.
- Events: `add_event_listener` handlers are `Function<fn(JsValue) -> Undefined>`
  (`EventMap[Type]` is an indexed access); `message_text` reads `data().as_string()`.
- `set_timeout_with_ms_delay(cb, ms)` → `(cb, ms, &[])` plus `::<JsValue>`;
  `block_concurrency_while::<JsValue>`.
- `Response::new_with_null_and_init(&Null::NULL, &init)` is generated (`67fa212`), so the port's
  hand-written overload is gone with `worker.d.ts`.
- `block_concurrency_while_promise::<JsValue>`, `first::<JsValue>()` → `JsNullable` → `JsValue`,
  `storage.get::<JsValue>("count")` in the `#[durable_object]` test class.
- Integer hints (second commit): `status(200.0)` → `status(200)`, `redirect_with_status(.., 302)`,
  `close_with_code_and_reason(1000, ..)`, `limit(1)`, `upload_part(u16)`, and the same in
  `Response::builder().status(..)` / `ext/`.
- `Headers.getAll` (`@throws {TypeError}`) and `entries`/`keys`/`values` (`@throws {never}`) that
  the port's `worker.d.ts` had moved into `annotations.d.ts`.

`test/src/util.rs` and the GAP comments are otherwise unchanged; the `IncomingRequestCfProperties`
accessors (`latitude`, `longitude`, `region`) resolve directly on the type now.

## Review follow-up

Independent review of the first 17 commits (`~/Projects/master/session/review-workers-types.md`):

- Applied by amending the commit: #1 (promotion only for declared object parts, with the branded
  and `T & {…}` fixture cases), #2 (`extends` dedupe, `Omit` dropped with a warning; `Exclude<T, …>`
  reads as `T` since it is a no-op on a non-union), #3 (per-overload body scope and type params),
  #4 (namespaced enums by value), #6 (defaults naming earlier params), #7 (`T[]` rule moved to
  codegen; mono mode keeps the slice), #8 (unhandled side kinds and ambiguous names error, seven
  unit tests, all `--annotate` files named in the bail message, module moved to
  `parse/annotate.rs`), #9 (the TypedArray-widening half dropped — `2d63fac` already landed it —
  leaving the float-literal fix with a CONVENTIONS line and a negative-literal fixture line).
- #5 is a new commit (`7c24895`) since the enum commit it addresses landed as `2d63fac`.
- Nits taken: #11 (declared members take precedence over the union's narrowing), #15 (fixture
  comment), #16 (AGENTS.md lists `surface.rs`), #17 (`prototype`-only fixture line), #18 (parity
  file corrected; listed under Gaps).
- Nits declined: #10 (an explicit `instance_type` field for the split-class pattern — a refactor
  of landed behaviour with no output change; recorded as a follow-up), #12 (positional construct
  signature return mapping — not needed by workers-types, documented as-is), #13 (structural
  bound matching in `widen_async_return_param` and the two wasm-bindgen limitations — upstream
  issues to file rather than ts-gen changes), #14 (recorded under Gaps; outside the inventory).
