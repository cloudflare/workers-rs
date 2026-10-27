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

Session 4, on `gbedford/workers-parity` at `104a370`:

- `ad5ee90` Inherited members: between sibling parents declaring the same member the earliest in
  `extends` order provides it (was: the last); the child's own redeclaration still wins over all.
  The rest of the multi-`extends` decision (flatten parents beyond the first, transitively, for
  interfaces and promoted intersections; first parent keeps `extends =`/`Deref`) had already
  landed as `98976e4`, and the `AsRef` detours were removed from `ext/` and the port in `5dca395`.
- `b21ce29` Types named like the Rust prelude (`Result`, `Option`, `String`, `Vec`, `Box`,
  `Default`, `From`, …) take the `js_sys` collision rename (`Foo_` + `js_name`) with no `pub use`
  back to the original name; compile-only integration test; `generic-defaults`' local `Box<T>`
  becomes `Box_<T>`.

Session 5, on `gbedford/workers-parity` at `b21ce29`:

- `5b9ba39` Overload expansions use their own overload's return type and scope: the `_promise`
  form of a non-primary expansion promised the first overload's return (`get_with_keys_promise`
  → `Promise<JsOption<T>>` instead of `Promise<Map<JsString, T>>`), and the cross-overload dedup
  rendered every expansion's parameters in the first overload's scope, which left a stray
  `use JsValue as Args;` in the workers-types output. Found while writing the inference fixture.
- `96cf3bf` The type-parameter inference rule (decided; see Decisions): `codegen/type_params.rs`,
  applied where class members and free functions enter codegen; fixture
  `uninferrable-type-params.d.ts` with the four cases; CONVENTIONS "Uninferrable method type
  parameters" under "In-scope generic type parameters". The `iterables`, `generic-slice` and
  `overload-returns` fixtures gain an argument where their point was the generic itself.
- `bd1ec01` Nested arrays: the inner array of `T[][]` is the `Array<U>` handle
  (`Vec<Array<JsString>>`, `&[Array<Number>]`); wasm-bindgen has no `Vec<Vec<_>>` ABI. Surfaced by
  `D1PreparedStatement.raw<T = unknown[]>(): Promise<T[]>` once `T` erases to its default.

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
- `f170c39` Submodule pointer for the two session-4 commits; the generated bindings are unchanged
  (the inventory has no prelude-named type and no sibling parents sharing a member).
- `0e93ad0` Regenerated for the inference rule (bindings 133+/154−): 12 turbofishes go in the
  test worker (`run`, `list`, `list_with_options`, `get_with_keys`, `get_with_metadata`, `raw`,
  `set_timeout_with_ms_delay`, `SqlStorageCursor.raw`); `sql.rs` casts each raw row with
  `unchecked_into::<Array>()` and `d1.rs` deserializes `Vec<Array>` rows. On the rebase onto
  `cb8cd7d` the new `ext/` surface lost three more (`KvStore`'s `get_with_metadata_…` calls,
  `Delay`'s `set_timeout_with_ms_delay`).
- `c9e4c10` Regenerated from the restacked ts-gen tip `2171ea7` (on main `a306021`); statics
  lower-cased, `cloudflare::*` module items re-exported flat, one ts-gen regression hand-patched
  (see "Restack regressions (session 11)").
- `8dacaa6` wasm-bindgen submodule on #5384 (`69d72955b`, ts-gen's pin) with #5375 rebased on top;
  `38d0d0d` regenerated from the restack-3 tip `5b2d7bf` (see "Restack 3 (session 13)").
- `a4cbe97` Regenerated without mono from the restack-5 tip `45a278c` (see "Restack 5 (session
  15a)").
- `54d15c2` wasm-bindgen submodule on #5390 (`67e24940f`) with #5375 rebased on top; `f8fb34a`
  regenerated with `--experimental-generic-mono` from `01ed819` (#122 on `45a278c`), with the
  `mono-patch` stopgaps and the `api-diff` script in `ts-gen-worker/tools/` (see "Generic mono
  (session 15)").

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
- **Method type parameters no argument infers: keep for a bare-`T` return, erase otherwise**
  (decided by Guy from the session-4 design). Kept when an argument mentions it as lowered (a
  variadic callback is the untyped `Function`, a `Promise<T>` callback return erases) or when the
  awaited return is exactly `T` / `T | undefined` / `T | null`, since there the turbofish is the
  cast (`get::<JsString>(key)`, `json::<Row>()`). Otherwise the parameter erases to the default of
  the type it instantiates in the return (`D1Result<T = unknown>` → `JsValue`, `Fetcher<T =
  undefined>` → `Undefined`; the type's default wins over the method's so `run()` and `batch()`
  agree on `D1Result<JsValue>`), else the method's own default, else `any`, and leaves the
  signature; `setTimeout<Args>` drops `Args` outright. `_promise` / `try_` companions share the
  signature. Applied at codegen entry on the cloned member list (`ClassConfig::from_*`,
  `emit_function`) rather than as a parse post-pass, because it reads the lowering of callbacks and
  the defaults of types that may be declared later or in other files.
- **A constrained, defaultless parameter erases to `any`, not its constraint.** `SqlStorageCursor
  .raw<U extends SqlStorageValue[]>(): IterableIterator<U>` is now `Iterator` (of `JsValue`),
  where TypeScript itself would infer the constraint. Using the constraint would be more precise
  but self-referential bounds (`waitForEvent<T extends Rpc.Serializable<T>>`) do not lower, so the
  decision's "else `JsValue`" stands; recorded as a possible refinement under Gaps.
- **`Request`'s `Cf` default is overridden from the side file** (`Cf =
  IncomingRequestCfProperties<CfHostMetadata>`, and the same on `RequestInfo` so
  `Fetcher.fetch(&Request)` accepts the bare type). Upstream's `CfProperties<…>` union erases, and
  a handler's `req.cf()` is the incoming shape; `worker` has one `cf()` too. The price: an outgoing
  `Request::new(..)` is typed with the incoming `Cf` as well, which is what `worker` does.
- **Prelude collisions rename without a re-export.** The `js_sys` scheme re-exports `Foo_` as
  `Foo` so the public path is unchanged; for a prelude name that alias would itself shadow
  `Result`/`String`/… for every binding in the module (explicit items beat the prelude exactly as
  they beat a glob), so the suffixed name is the public one. The reserved list is everything the
  Rust 2021 prelude puts in the type namespace (types, traits, `Some`/`None`/`Ok`/`Err`); the
  variants are harmless as types but cost nothing to include and keep the rule mechanical.
  `Error`/`Iterator` were already in the `js_sys` list; `Self` is a keyword `make_ident` already
  suffixes.
- **Sibling parents: first in `extends` order wins.** TypeScript rejects incompatible
  redeclarations across siblings, so this only fixes the output order and the pick among identical
  ones; the child's own redeclaration remains the override.
- **`console` stays a `thread_local_v2` static** (`console.with(|c| c.log(..))`), the honest
  lowering of `declare const console: Console`; a `console_log!` ergonomic belongs in `ext/`.
- **The wasm-bindgen pin follows ts-gen's.** ts-gen main emits wasm-bindgen#5384's slice modes,
  which main `9bec40cf3` rejects, so the submodule moved from main to `69d72955b` (the revision
  ts-gen's integration tests pin), not the newer `copy-slice` tip. Hand-patching ~70
  `slice_to_array` attributes back in would have been the alternative.

## Open problems

- **64-bit integers carried as JS numbers.** `worker` exposes `R2Object.size`, KV
  `expiration`/`expirationTtl`/`cacheTtl`/list `limit`, `getAlarm`, `File.lastModified` as
  `u64`/`i64`; here they are `f64`. Proposal: `@rust u64` on a `number` emits the `f64` extern
  binding under a private name plus an inline wrapper converting with `as` (both directions),
  exact up to 2^53. ts-gen already emits non-extern code (factories, builders, `PartialEq`), so
  the reviewer considers the "no shims" blocker weak; needs Guy's call on the policy.
- **`--annotate` targets by bare name.** A name declared both globally and in a module is now an
  error rather than first-wins; honoring `declare module "x" { … }` / `declare namespace` in the
  side file as scopes is the fix when a case appears.
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
- ts-gen: two inline option literals differing only in a literal (`raw({columnNames: true})` /
  `raw({columnNames?: false})`) synthesize `D1PreparedStatementOptions` and `…Options2`.
- ts-gen: a required nullable dictionary field (`tag: string | null`) has no way to be set to
  `null` through the factory.
- ts-gen: enums and discriminated unions do not consult `renamed_locals`, so a string enum named
  `Global` or `Option` would still collide (class-like types only, same as the `js_sys` rename).
- ts-gen: `wasm_bindgen::prelude` names (`JsValue`, `JsCast`, `Closure`, `JsError`) are not in
  either reserved list; a declaration named so would shadow them like a `js_sys` name.
- ts-gen: a declared `class String` resolves to the lib builtin, so a return of that class emits
  the bare prelude `String` (kept out of the `prelude-collision` fixture; no such declaration in
  the inventory).
- ts-gen: the surface walk follows type-parameter constraints, so `--export Service` emits ~1,900
  lines nothing references (bug 4 under "Restack 3 (session 13)").
- ts-gen: a construct signature's concrete return arguments are dropped: `new (UnderlyingByteSource,
  …): ReadableStream<Uint8Array>` returns `ReadableStream` (non-mono) or an unconstrained
  `ReadableStream<R>` (mono).
- ts-gen: an erased type parameter with a constraint but no default could erase to the constraint
  (TypeScript's own inference) instead of `any`, when the constraint does not mention the
  parameter; would give `SqlStorageCursor.raw()` → `Iterator<Array>`.

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

## Migration-table deltas from the rebased stack (for workers-rs-4 / MIGRATING.md)

The ts-gen stack now sits on upstream #51 (`gbedford/nullability`), whose argument rule replaces
the one the port was written against. Every binding argument whose TS type admits an empty now
spells which empty:

| TS argument | Rust | pass a value / the empty |
| --- | --- | --- |
| `x?: T` / `T \| undefined` | `Option<&T>` | `Some(&v)` / `None` |
| `T \| null` | `&JsNullable<I>` | `&JsNullable::wrap(v)` / `&JsNullable::new()` |
| `T \| null \| undefined` | `Option<&JsNullable<I>>` | `Some(&JsNullable::wrap(v))` / `None` |

`I` is the JS spelling of `T` (`JsString` for `string`, `Number`, a bound class). Returns are
unchanged (`Option<T>`). Where this lands in the inventory, and how the port and `ext/` moved
(`a369845`):

- `Response::new_with_str_and_init(body, init)` and the other `BodyInit | null` constructors:
  `&JsNullable::wrap("text".into())`; an empty body is `&JsNullable::new()` (the former
  `new_with_null_and_init(&Null::NULL, ..)` is gone). `ext/`'s `Response::builder().ok(..)` /
  `.empty()` / `.body(..)` / `.error(..)` hide this for `worker`-style code.
- `RequestInit::builder().signal(&JsNullable::wrap(signal))`, `.body_with_str(&JsNullable::wrap(
  s.into()))`; `ResponseInit::builder().web_socket(&JsNullable::wrap(ws))`.
- `R2Bucket::put_with_str(key, &JsNullable::wrap("v".into()))`, `put_with_typed_array(key,
  &JsNullable::wrap(bytes.into()))`, `put(key, &JsNullable::wrap(stream))`; a null body is
  `put_with_str(key, &JsNullable::new())`.
- `KVNamespaceListOptions::builder().prefix(&JsNullable::wrap("p".into()))` / `.cursor(..)`
  (upstream declares them `string | null`); `ext/`'s `KvStore` list builder takes `impl AsRef<str>`
  and wraps.
- `clear_timeout(&JsNullable::wrap(Number::from(id)))` (`timeoutId: number | null`).
- Optional (`?:`) dictionary fields keep plain setters (`&T`); only fields written `T | undefined`
  take `Option<&T>` — `worker`'s `EmailAttachment::new_inline` now takes `content_id:
  Option<&str>` because the other branch leaves it `?: undefined`.

Also from the rebase: string enums in unions are enum payload variants (`Iso3166Alpha2Code(
Iso3166Alpha2Code)`), which needs wasm-bindgen#5376 — the `wasm-bindgen` submodule carries that
commit (`1ea319f65`) on top of the `reinit = "reconstruct"` branch; it is local until that branch
is pushed. The regenerated `bindings.rs` is 561 lines shorter than before the rebase.

## Restack regressions (session 11)

Bindings regenerated from the restacked tip (ts-gen `2171ea7` on main `a306021`, `c9e4c10`).
`chomp test:ts-gen-worker` and `-http`: 194 passed / 4 skipped each (baseline unchanged);
`worker`'s own `chomp test`: 145 passed / 4 skipped. No spec went red; nothing was loosened.
`bindings.rs` diff: 647 lines, every hunk below.

| # | Change (old → new) | Cause | Port change | Class |
|---|---|---|---|---|
| 1 | `EmailAttachment::new_inline*` / `new_attachment*` bodies: `inner.set_disposition("inline")` against `set_disposition(val: DispositionKind)`; does not compile, in `bindings.rs` **and** `worker/src/bindings/email.rs` | main #39 `88d0863` (enum setters by value) meets the discriminated-union branch factory, which still bakes the raw literal; present on main's own `workers-types` snapshot | hand patch, marked `// ts-gen regression:`: `bindings.rs` 19146, 19164, 19182 (`Inline`), 19225, 19241, 19257 (`Attachment`); `email.rs` 201, 219, 237, 280, 296, 312 | **regression** (ts-gen bug 1) |
| 2 | `WebSocket`, `RequestInitCfProperties`, `IncomingRequestCfPropertiesBase`, `CloudflareAccessIdentity`: `derive(PartialEq, Eq)` → `derive(PartialEq)` | main #81 `70e8b95` `derive_eq` (first parent written with TS args: `EventTarget<WebSocketEventMap>`, `Record<string, unknown>`), kept for non-generic types by the restack | none (nothing in the port uses `Eq`) | **regression**: `worker::WebSocket` is `Eq`; a user struct deriving `Eq` cannot hold one (ts-gen bug 2) |
| 3 | `pub static CONSOLE/CRYPTO/CACHES/PERFORMANCE` → `console/crypto/caches/performance` with `#[allow(non_upper_case_globals)]` | main #45 `2191f49` | ~20 call sites renamed (`ext/console.rs`, `lib.rs`, `test/src`); a crate-level `allow` on `mod bindings` while the pin predated #5377, removed in `db26b8b` | neutral drift |
| 4 | `connect`, `connect_with_*` → `cloudflare::sockets::…`; `EmailMessage` → `cloudflare::email::EmailMessage` (both crates; `ForwardableEmailMessage extends = cloudflare::email::EmailMessage`) | main #94 `76a5d01`; roots closure `b1314e1` places module declarations in their nested `pub mod` | `pub use` re-exports in `ts-gen-worker/src/lib.rs` and `worker/src/email.rs`; flat spelling unchanged at every call site | neutral drift |
| 5 | `pub type Cloudflare` (`compatibilityFlags`) gone; `pub mod cloudflare` gains the `Cloudflare` namespace's `Env`, `GlobalProps` and the `GlobalProp`/`MainModule`/`Exports` aliases | `e011e0f` (closure through namespace owners): `Cloudflare.Exports` now reaches the namespace, not the merged interface; #88/#94 share one `pub mod` | none (unused; `exports()` was and is `JsValue`) | neutral drift (not a root) |
| 6 | string-enum setters and builder methods `&str` → enum by value: `Response::set_type(TypeKind)`, `ResponseInit` / `RequestInit` `encode_body(EncodeBodyKind)`, `cache(CacheKind)`, `WebSocket::set_binary_type`, `EmailAttachment::set_disposition`, cf `polish`/`grpc_web`, image `fit`/`gravity`/`trim`/`flip`/`format`/…, container stdio | main #39 | `.encode_body("manual")` → `.encode_body(EncodeBodyKind::Manual)` (2 sites) | improvement (typed; breaking for `&str` callers) |
| 7 | mixed-union setters renamed with the type: `set_country_with_js_value(&str)` → `set_country_with_country_kind(CountryKind)`; likewise `stdin`, `repeat`, `quality` (and builder methods) | main #39 enum payloads + #80 `eaa84b2` setter routing | none | improvement (typed; breaking rename) |
| 8 | `set_rotate(f64)` → `set_rotate(RotateKind)` (`0 \| 90 \| 180 \| 270 \| 360`) | main #39 / #84 | none | improvement |
| 9 | `Array<T>` → `Vec<T>` returns and `&Array<T>` → `&[T]` arguments over a type parameter: `D1Result::results`, `KVNamespaceListResult::keys`, `MessageBatch::messages`, `SqlStorageCursor::to_array`, their factories | `809a44e` dropped; main #96 `7e3d3dd` | `.keys().length()` → `.keys().len()` (3 sites) | neutral drift |
| 10 | `slice_to_array` dropped on variadics and handle slices (`console.*`, D1 `bind`/`batch`, R2 `complete`, `Blob`/`File` bits, `AbortSignal.any`, email attachments/recipients); added on `&[f64]` (`AnalyticsEngineDataPoint::set_doubles`) | main #96 | none; D1 batch/bind and R2 multipart specs confirm handle slices still cross as an `Array` on the pinned wasm-bindgen | neutral drift; `set_doubles` improvement (`Array`, not a `Float64Array` view) |
| 11 | `IncomingRequestCfProperties::bot_management() -> JsValue` → `IncomingRequestCfPropertiesBotManagementBase` (also the enterprise variant and every factory) | main #81 (member-position intersection bounded to its most specific member) | none | improvement (enterprise `ja3Hash` reachable only by cast) |
| 12 | `#[wasm_bindgen(private)]` on every generated enum | main #39 | none | neutral drift |

### ts-gen bugs found

1. **Discriminated-union branch factory passes a literal to an enum setter** (on main). Input:
   ```ts
   type Att =
     | { disposition: "inline"; id: string }
     | { disposition: "attachment"; name: string };
   declare function send(a: Att): void;
   ```
   Output: `set_disposition(this: &Att, val: DispositionKind)` but `new_inline` does
   `inner.set_disposition("inline")`. The interface-field form (`certRevoked: "1" | "0"`) already
   bakes `CertRevokedKind::_1`; the branch factory needs the same literal → variant mapping.
2. **`derive_eq` drops `Eq` where the parent has it.** Input:
   ```ts
   declare class Target<M = {}> { on(): void; }
   interface Sock extends Target<{ a: string }> { send(): void; }
   declare var Sock: { prototype: Sock; new (): Sock };
   interface Props extends Record<string, unknown> { a?: string }
   declare function f(p: Props): Sock;
   ```
   Output: `Sock` and `Props` derive `PartialEq` only, while `declare class Sock extends
   Target<{…}>` derives `PartialEq, Eq`. `Target<M>` carries the handle-identity `Eq` from the
   generics layer, so #81's reason for dropping `Eq` no longer holds. Before the restack these
   types derived `Eq` and compiled.
3. **A branch admitting several literals breaks the merged discriminant's enum** (on main):
   `type S = { kind: "open" } | { kind: "close" | "error" }; declare function f(s: S): void;`
   merges `kind` as a nested union and emits `enum KindKind { Open = "open", JsValue(String) }`
   plus `set_kind_with_str("close")`, instead of one enum with three variants.

Bug 1 has a fix on ts-gen `gbedford/branch-factory-enum-literal` (`0415dee`, on main, under review).

Not ts-gen: `~/.cargo/bin/ts-gen` is shared by every worktree, so whichever agent ran
`cargo install` last decides what everyone generates with. Reinstall from this worktree's
`ts-gen/` before every regeneration (sessions 11 and 12 did).

### wasm-bindgen pin (session 12)

`db26b8b`: the submodule is wasm-bindgen `origin/main` `9bec40cf3` (#5376, #5377, #5381, #5383)
with #5375's branch rebased on top as local `gbedford/reinit-reconstruct` = `41c895974` (not
pushed; origin's branch is still `870389dd3`). Rebase record:

- `6f84fd3f3` (re-bless stale reference fixtures): dropped, already upstream.
- `262626e26` → `c8fc01078` (export call guard before argument conversion), `binding.rs`: main
  #5350 moved body generation into `BodyParams::generate`, which reruns when argument names
  shadow; the hoisted `__wbg_call_guard();` moved there, after the subclass sentinel and before
  the receiver's stale check. The `wrap_hoisted_export_call` hunk merged with main's `js.local`.
- `88b506e56` → `7ced3643a` (parse `reinit`): main already set `SCHEMA_VERSION = "0.2.130"`, so
  only `APPROVED_SCHEMA_FILE_HASH` moves (`17908788602150006240`); message amended to say so.
- `870389dd3` → `41c895974` (reconstruct): `BodyParams::generate`'s inline stale-object prelude
  becomes `js.assert_not_stale("this")`; CHANGELOG keeps main's #5376 entry and #5375's.

Checked on the rebased branch: each commit `cargo check`s (`cli-support`, `macro-support`,
`shared`); `schema_version`; reference tests `panic_unwind{,_legacy}`, `reinit_reconstruct`,
`targets` (+ `_atomics`, `_mvp`), `import`; `reinit*` / `termination*` CLI tests: 10 pass, and the
4 that fail (`termination`, `termination_reset_state`, `termination_abort_handler_unwind_{panic,
abort2}`) fail identically on `origin/main` in this environment. Macro UI tests and the rest of
the CLI suite not run (disk).

Workspace: bindings regenerate byte-identical; the `allow` on `mod bindings` is gone (#5377);
fmt, `cargo check --locked` (no `Cargo.lock` change), clippy, `chomp test:ts-gen-worker` /
`-http` 194 passed / 4 skipped, `chomp test` 145 / 4. With #5383 the output compiles as is:
non-generic types' `derive(PartialEq, Eq)` and the generic types' manual handle-identity impls
coexist (the manual impls become redundant once ts-gen derives everywhere).

### Migration-table deltas (for workers-rs-4 / MIGRATING.md)

- Global statics: `console`, `caches`, `crypto`, `performance` (lower-case, like JS);
  `console.with(|c| c.log(..))`, `caches.with(CacheStorage::default)`. The `console_log!`-style
  macros are unchanged.
- `connect`, `EmailMessage`: unchanged spelling at the crate root (re-exported); the canonical
  paths are `cloudflare::sockets::connect` and `cloudflare::email::EmailMessage`.
- String-enum options take the enum, not a string: `.encode_body(EncodeBodyKind::Manual)`,
  `.cache(CacheKind::NoStore)`, `set_binary_type(BinaryTypeKind::Arraybuffer)`,
  `EmailAttachment::set_disposition(DispositionKind::Inline)`; mixed setters are named after the
  type (`set_country_with_country_kind(CountryKind::…)`).
- `D1Result::results()`, `KVNamespaceListResult::keys()`, `MessageBatch::messages()`,
  `SqlStorageCursor::to_array()` return `Vec<T>` (`.len()`, slice iteration) instead of
  `Array<T>`.
- `WebSocket` is `PartialEq` but no longer `Eq` (bug 2 above).

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

## Restack 3 (session 13)

Bindings regenerated from ts-gen `5b2d7bf` (#61–#69 on #51, #52, #155, #53, #54 over main
`2796dd3`), `38d0d0d`; the previous tip `b0d7ef4` gave byte-identical output (its inline-wrappers
doc fix changes nothing here). The output targets wasm-bindgen#5384, so `8dacaa6` first moves the
submodule to `69d72955b` with #5375's three commits rebased on top (only the approved schema hash
moved, now `10903523727203315678`; the old bindings build and pass on it unchanged).
`chomp test:ts-gen-worker` and `-http`: 194 passed / 4 skipped each; `chomp test`: 145 / 4
(baselines unchanged). Nothing loosened. `bindings.rs` 20,329 → 21,955 lines (+2,086 / −460).

| # | Change (old → new) | Cause | Port change | Class |
|---|---|---|---|---|
| 1 | 38 generic types (`Request`, `ReadableStream`, `ExecutionContext`, `DurableObjectStub`, `KVNamespaceListResult`, `D1Result`, `IncomingRequestCfProperties`, …): manual `impl PartialEq` / `impl Eq` → `derive(PartialEq, Eq)` | #63 on wasm-bindgen#5383 (derive compares handles, no bound on the type parameters) | none | improvement (same semantics, ~300 lines less) |
| 2 | `WebSocket`, `RequestInitCfProperties`, `IncomingRequestCfPropertiesBase`, `CloudflareAccessIdentity`: `derive(PartialEq)` → `derive(PartialEq, Eq)` | #63 (same fold) | none | improvement: resolves session 11 row 2 / bug 2 |
| 3 | `slice_to_array` dropped on the 76 `&[String]` arguments (`console.table`, storage `get`/`delete` keys, `acceptWebSocket` tags, SubtleCrypto key usages, `R2Bucket.delete`, `WebSocket::new_with_slice`, `Container.exec`, `string[]` setters) and `email.rs`'s `set_to/cc/bcc_with_slice` | main #104 / #118: wasm-bindgen#5384 passes a borrowed slice element by element as a JS array | wasm-bindgen pin (`8dacaa6`) | neutral (same JS array; needs #5384) |
| 4 | `AnalyticsEngineDataPoint::set_doubles(&[f64])`, `set_detection_ids(&[u32])`: `slice_to_array` → `#[wasm_bindgen(slice = "array")]` on the argument | main #104 | pin | neutral |
| 5 | KV bulk `get_with_array*(&Array<JsString>)` → `get_with_slice*(&[String])` (`get`, `getWithMetadata`, their text/json wrappers and `_promise` forms) | main #113 (`Array<T>` = `T[]`) on the erased `Key extends string` | `ext/kv.rs` `KvGetBulkBuilder` holds a `Vec<String>`; test `get_bulk_text` passes `&[String]` | improvement (no `Array` building; breaking rename) |
| 6 | `CloudflareAccessIdentity::groups() -> Option<Array<Object>>` → `Option<Vec<Object>>`, setter / builder `&[Object]` | main #113 | none | neutral drift |
| 7 | `PerformanceMark::new`, `new_with_maybe_options` gone | #155: a class reached only by reference (`Performance.mark()`'s return) is emitted as a type, without its constructor | none (unused; `--export PerformanceMark` restores it) | neutral (by design) |
| 8 | `pub mod cloudflare` loses `Env`, `GlobalProps`, `GlobalProp`, `MainModule`; `Exports = JsValue` stays | #155 closure (does not enter the erased mapped type) | none | neutral: reverts session 11 row 5's unused additions |
| 9 | new, ~1,900 lines: `ExportedHandler` (+ builder) and its 8 handler aliases, `EmailExportedHandler`, `TestController`, `TraceItem` and 20 `Trace*` types, `ScriptVersion`, `TracePreviewInfo`, `pub mod rpc` (3 branded types), `pub mod tail_stream` (36 types), `SqlStorageValue = JsValue`, root enums `EventKind`, `GetWebSocketEventKind`, `InfoKind`, `OnsetInfoKind`, `SpanOpenInfoKind`, `MessageKind`, `LevelKind`, `ValueKind` | #155's surface walk follows type-parameter constraints: `type Service<T extends … \| ExportedHandler<any, any, any> \| undefined>` (itself `pub type Service = JsValue`), `SqlStorage.exec<T extends Record<string, SqlStorageValue>>` | none | neutral drift: unreachable from anything emitted (ts-gen bug 4) |
| 10 | `pub type EventListener<EventType>` → `EventListener<EventType = Event>` | #63 (alias parameters carry defaults) | none | improvement |

No other hunks. The `cloudflare::email::EmailMessage` collapse, the `--export` roots and every
other #155 path produce the session-12 output unchanged.

### Resolved since session 11

- Session 11 row 2 / bug 2 (`derive_eq` dropping `Eq`): resolved (rows 1–2); the session-12 note
  that the manual impls would become redundant is done.
- Session 11 row 5 (`Cloudflare` namespace items): gone again (row 8).
- Session 11 row 10 (`set_doubles` `slice_to_array`): now `slice = "array"`, same `Array`.
- Gaps (removed from the list): the namespaced dynamic-union payload (`OnsetInfoKind` now holds
  `tail_stream::FetchEventInfo`, and row 9 makes it compile in this crate), `Vec<f64>` /
  `Vec<bool>` payloads (`ValueKind { VecOfF64(Vec<f64>), VecOfBool(Vec<Boolean>) }` builds on
  #5384), and `(string | null)[]` (`TraceEventInfo::traces() -> Vec<JsNullable<JsString>>`).
- Still open: bug 1 (branch factory literal; the 12 `// ts-gen regression:` patches are
  re-applied, the fix being ts-gen #157 outside this stack) and bug 3 (multi-literal branch:
  `enum KindKind { Open, JsValue(String) }` plus `set_kind_with_str("close")`, rechecked on
  `5b2d7bf`).

### ts-gen bugs found

4. **The surface walk follows type-parameter constraints, even of an alias that erases**
   (#155, `type_params` in `codegen/surface.rs`). Input:
   ```ts
   interface Handler { fetch?(): void; }
   interface Fetcher<T = undefined> { go(): void; }
   type Service<T extends Handler | undefined = undefined> =
     T extends Handler ? Fetcher<T> : Fetcher<undefined>;
   ```
   `--export Service` emits `pub type Service = JsValue` and `Handler`, which nothing emitted
   names (and not `Fetcher`, which the body does name). A constraint never reaches the Rust output
   (a kept parameter is `T: JsGeneric = Default`; primitive-constraint erasure emits nothing), so
   walking only defaults would drop row 9 without losing a reference. Proposal: drop
   `p.constraint` from the walk.

### Migration-table deltas (for workers-rs-4 / MIGRATING.md)

- KV bulk get: `kv.get_with_slice(&keys)` with `keys: &[String]` (was
  `get_with_array(&Array<JsString>)`); `ext/`'s `get_bulk(&[impl AsRef<str>])` is unchanged.
- `WebSocket` (and the other three in row 2) are `Eq` again, so a user struct deriving `Eq` can
  hold one, as with `worker::WebSocket`.
- `PerformanceMark` has no constructor in this surface; `performance.mark(name)` creates one.

## Restack 5 (session 15a)

Baseline before generic mono: bindings regenerated without mono from ts-gen `45a278c` (the stack
rebased onto main `f8466d4`, i.e. #155 as landed plus #157), wasm-bindgen pin unchanged
(`a8aff4642`). `chomp test:ts-gen-worker` and `-http`: 194 passed / 4 skipped each (one `-http`
run timed out `r2 > delete` at 5 s; the rerun and three isolated `r2.spec.ts` runs pass);
`chomp test`: 145 / 4; clippy clean; benchmark `.wasm` / glue the same size as at `9439547`
(120,645 / 33,871 B). Nothing loosened. `bindings.rs` 21,955 → 22,148 lines (+206 / −13);
`email.rs` −6.

| # | Change (old → new) | Cause | Port change | Class |
|---|---|---|---|---|
| 1 | `set_timeout(cb, args)` (variadic), `set_timeout_with_ms_delay(cb, ms, args)` → `set_timeout(cb)`, `set_timeout_with_function_and_ms_delay(cb, ms)`, `set_timeout_with_function_and_args(cb, args)`, `set_timeout_with_function_and_ms_delay_and_args(cb, ms, args)` | main #155 as landed: a function's declarations are its overloads, so the non-variadic `declare function setTimeout(callback, msDelay?)` now expands; before, only the later generic variadic one survived | `ext/delay.rs`, `test/src/util.rs`: `set_timeout_with_function_and_ms_delay(cb, ms)`, no `&[]` | improvement (a non-variadic form); breaking renames, and `_with_function_` is noise (both overloads' callbacks lower to `&Function`, yet the suffix still names it) |
| 2 | `ReadableStream::new_with_underlying_source_and_strategy` → `…_and_queuing_strategy`; new `new_with_underlying_byte_source{,_and_queuing_strategy}`, and the types they reach: `UnderlyingByteSource` (+ builder), `ReadableByteStreamController`, `ReadableStreamBYOBRequest` | #155: every construct signature of `declare const ReadableStream: { new (UnderlyingByteSource, …); new <R>(UnderlyingSource<R>, …) }` is a constructor overload; before, the later one replaced the first | none (unused) | improvement (byte streams constructible); the byte form returns `ReadableStream` (`<JsValue>`), not the declared `ReadableStream<Uint8Array>` (gap) |
| 3 | `ForwardableEmailMessage::reply_with_builder{,_promise}` docs: its own overload's JSDoc instead of `reply(message)`'s | #155 (each overload keeps its own JSDoc) | none | improvement |
| 4 | the 12 `// ts-gen regression:` hand patches (6 in `bindings.rs`, 6 in `email.rs`) are gone; the factory emits `set_disposition(DispositionKind::Inline)` itself | main #157 | patches dropped | improvement: resolves session 11 bug 1 |

No other hunks.

### Migration-table deltas (for workers-rs-4 / MIGRATING.md)

- `setTimeout`: `set_timeout_with_function_and_ms_delay(&cb, ms)` (was
  `set_timeout_with_ms_delay(&cb, ms, &[])`); the variadic forms are `…_and_args`. `Delay` is
  unchanged.

## Generic mono (session 15)

`build:ts-gen-worker:types` passes `--experimental-generic-mono` to all three generations; ts-gen is
`01ed819` (`workers-mono-test`: the stack tip `45a278c` with #122's five commits). The full
reading — patterns, interesting cases, every port call site, sizes, feedback with reproducers — is
`generic-mono.md`; the signature-level diff is `generic-mono-api-diff.md` (script in
`ts-gen-worker/tools/`). `worker/src/bindings/email.rs` stays non-mono and is byte-identical.

### wasm-bindgen pin

`54d15c2`: the submodule is wasm-bindgen `67e24940f` (#5390 on #5384, #122's pin) with #5375's
three commits rebased on top as local `gbedford/reinit-reconstruct-5390` = `ea363f37e` (not
pushed). `67e24940f` descends from the previous base `69d72955b`, so the rebase is `--onto
67e24940f 69d72955b`:

- `001a19b0b` → `39bb0617f` (export call guard): clean.
- `99db02f77` → `f600be2ef` (parse `reinit`): conflict only in `APPROVED_SCHEMA_FILE_HASH`;
  #5390 changed the schema files, so the hash is recomputed (`16382874117616585400`).
- `a8aff4642` → `ea363f37e` (reconstruct): clean.

Each commit `cargo check`s (`cli-support`, `macro-support`, `shared`); `schema_version` passes; CLI
tests filtered on `reinit`, `panic_unwind`, `targets`, `_import_rs`, `termination`: 22 pass and the
same 4 `termination*` as session 12 fail (environment). The non-mono bindings build and pass on
the new pin unchanged (`chomp test:ts-gen-worker` / `-http` 194 / 4, `chomp test` 145 / 4).

### `js_sys_unstable_apis`: not enabled

The mono output does not need it: the generated code fails with the same errors with and without
the cfg, and builds both ways once patched. It also cannot be enabled: crates.io
`serde-wasm-bindgen 0.6.5` and `wasm-streams 0.7.0` stop compiling under it, both sit in `worker`'s
and `ts-gen-worker`'s dependency graphs, and a `--cfg` applies to the whole build (cargo has no
stable per-package rustflags), so scoping it to the ts-gen-worker crates would mean separate cargo
invocations and target directories for a cfg nothing uses.

### Stopgap patches

As emitted, `bindings.rs` has 499 errors at 113 sites from five ts-gen causes (none from #122;
generic-mono.md feedback 1–5). `ts-gen-worker/tools` `mono-patch` (run by the chomp task after
generation, then `rustfmt`) makes 55 span-located edits, each marked `// ts-gen mono: …
(generic-mono.md #N)`: `JsGeneric` on type parameters inside `Function<…>` (33 extern fns), the
missing builder/factory impl bounds (16 impls), two `_js_string` twins naming an erased parameter
removed, two colliding inline-wrapper twins renamed, `'static` on two inline wrappers. The task
regenerates the committed file byte for byte.

### Result

fmt, `cargo check --locked`, the clippy line, `chomp test:ts-gen-worker` and `-http` (194 passed /
4 skipped each), `chomp test` (145 / 4), `chomp build:bench`: all green. No spec changed. Port: 16
call sites for mono, 12 for clippy's `needless_borrows_for_generic_args` (generic-mono.md (d)).

| # | Change (old → new) | Cause | Port change | Class |
|---|---|---|---|---|
| 1 | `&str` arguments → `impl JsStringLike` (853 items) | mono | 12 clippy-driven `&x` → `x` | neutral / improvement |
| 2 | `JsGeneric` bounds dropped (479) | mono | none | improvement once it compiles; 33 sites do not (bug 1) |
| 3 | `&[String]` → `&[impl JsStringLike]` (117) | #122 | none | improvement; `&[]` needs a type |
| 4 | 385 `_js_string` twins (367 `JsString`, 18 `Array<JsString>` from #122) | mono / #122 | none | neutral; surface growth, `type__js_string` naming |
| 5 | generic-typed arguments `&T` → `T` (41): `Queue::send`, `write_with_chunk`, builder `cf`, `MessageSendRequest::new`, `read_at_least` | mono | `send(body)`, `write_with_chunk(chunk)` / `(… .into())` ×7, `init.cf(cf)` | **regression**: moves, and a borrow silently instantiates `T = &X` |
| 6 | constructors generic over the class's parameters (16): `TransformStream::new<I, O>()`, `Request::new_with_str<CfHostMetadata, Cf>`, `ReadableStream::new<R>()` | mono | `let t: TransformStream<Uint8Array, Uint8Array> = TransformStream::new()?` ×3 | **regression**: annotations needed, not tied to arguments |
| 7 | `impl JsStringLike` makes turbofish count a hidden parameter | wasm-bindgen | `get::<JsValue, _>` ×3 | **regression** |
| 8 | `KVNamespaceGetOptions<JsString>` → `<String>`, builder takes `Type` by value (24 items with `String` type arguments) | mono | `builder(value_type.to_string())` | neutral |
| 9 | `number \| bigint` and `string \| number \| boolean` → `impl JsBigIntOrNumberLike` / `JsBooleanOrNumberOrStringLike`; 15 `_with_big_int` / `_with_f64` / `_with_str` overloads removed | mono | none (`f64` still passes) | improvement |
| 10 | `Option<&str>` → `Option<impl JsStringLike>` (7) | mono | none | **regression**: `None` needs `None::<&str>` |
| 11 | `TypedArray` builder methods gain `where for<'a> &'a T: IntoWasmAbi` (22) | mono | none | neutral |
| 12 | `ValueKind::VecOfBool(Vec<Boolean>)` → `Vec<bool>` | #122 | none | improvement |

Sizes against `9439547` (benchmark worker, `-Oz`): `.wasm` 120,645 → 120,942 B (+0.2%; gzip −24 B),
glue 33,871 → 35,064 B (+3.5%; gzip +66 B), 77 imports both. Debug test worker glue +10.1%, of which
three quarters is the per-shim `// generic import` comment. `ts-gen-worker` compiles ~20–28% slower.

### Issues to raise

From #122: none that breaks the build. Two ergonomic notes: `&[]` for a `&[impl JsStringLike]`
parameter needs `&[] as &[&str]` (generic-mono.md feedback 9), and 18 more `_js_string` twins
(feedback 11).

Pre-existing mono, ts-gen main (reproduce on `f8466d4`): feedback 1 (unbounded parameters inside
`Function<…>`), 2 (impl bounds for `&[T]` setters), 6 (constructor type parameters), 7 (by-value
generic arguments), 9 (`None` for `Option<impl JsStringLike>`), 10 (`type__js_string`), 11 (twin
volume).

Pre-existing mono × stack layers (do not reproduce on main): feedback 2 for `Option<T>` (`2d6cfaf`),
3 (`_js_string` twin over a primitive-erased parameter, `592db90`), 4 and 5 (inline wrappers'
twins and bounds, `49afaaf`).

wasm-bindgen: feedback 8 (turbofish arity with argument-position `impl Trait`) and 12 (shim comment,
one shim per string type).

### Migration-table deltas (for workers-rs-4 / MIGRATING.md)

Only if the port adopts mono:

- Generic constructors need the type: `let t: TransformStream<Uint8Array, Uint8Array> =
  TransformStream::new()?;` (or `TransformStream<JsValue, JsValue>` for the untyped stream).
- Generic-typed arguments move: `queue.send(body)`, `writer.write_with_chunk(chunk)`; an untyped
  writer takes `JsValue` (`Uint8Array::from(bytes).into()`).
- Turbofish counts the string argument: `storage.get::<JsValue, _>(key)`,
  `stmt.first_with_col_name::<JsValue, _>(col)`.
- KV options: `KVNamespaceGetOptions::builder("text".to_string())` is a `KVNamespaceGetOptions<String>`.
- String arguments accept `&str`, `String`, `JsString` alike; `None` / `&[]` need a type.

## Restack 6 (session 16)

Both submodules repinned: ts-gen `gbedford/mono-prototype` `dc33537` rebuilt on main `48098a9`
under the standalone option-null-reads PR branch (`ts-gen-prs.md` "Restack 4"), wasm-bindgen
`gbedford/mono-prototype` `3a62aa7d4` on main `39ae823bb` with #5428 and #5427 under the rest
(`mono-prototype-patches.md` "After the session-16 restack"). Two regenerations: the first on
ts-gen main `b0af305` with the layer's first form (`T | null` reads `Option<T>`, both-empties
reads `Option<Nullable<T>>`, 97 → 16 `Nullable<`), the second on `48098a9` with the final rule
(every empty-admitting read `Option<T>`): `bindings.rs` 18,240 → 18,805 lines, 97 → 10
`Nullable<` (the 10 KV bulk `Map<String, Nullable<_>>` type arguments), 0 `Option<Nullable<`, 0
`Promise<Nullable<`. `chomp test:ts-gen-worker` and `-http`: 195 passed / 4 skipped each; `chomp test`: 145 / 4; `chomp lint:ts-gen-worker` clean (both regenerations). Benchmark worker (`-Oz`, first regeneration): `.wasm` 122,741 B
(gzip 51,560), glue 37,076 B (gzip 6,366), against 120,942 / 35,064 recorded in session 15 (+1.5% /
+5.7%; the glue grows with the `_with_big_int` and `_with_null` arms the port does not call).

| # | Change (old → new) | Cause | Port change | Class |
|---|---|---|---|---|
| 1 | every empty-admitting read is `Option<T>` (87 sites: `KVNamespace::get_text` → `Option<String>`, `Response::body` → `Option<ReadableStream<Uint8Array>>`, `web_socket`, R2 `get`, `metadata`, awaited `Promise<Option<_>, E>`, and the 5 `?: T \| null` dictionary getters) | option-null-reads PR + wasm-bindgen#5428 (`Option<T>` reads `null` as `None`) | the 13 `.into_option()` adaptations from the first regeneration dropped again; `kv.rs` `metadata()` back to `.map(..).transpose()` | improvement: restores the prototype's shape through main's carriers |
| 2 | top-type reads are bare `JsValue` under any empties (`Response::cf` / `ResponseInit::cf`, `deserialize_attachment(): any \| null`, `PerformanceMarkOptions::detail`, `AbortSignal::onabort`, `CryptoKey::algorithm`; were `Option<JsValue>`); other erased reads keep `Option<JsValue>` (`TraceItem::event`, `SpanOpen::info`) | #188's rule applied to every empty; the per-mono `cf?: any → Option<JsValue>` exception dropped | `response.rs` tests `cf.is_undefined() \|\| is_null()`; tests drop `.unwrap_or_default()` / `.and_then` on `cf()` and the attachment | neutral: `JsValue` carries both empties in-band |
| 3 | union-alias reads are one shared enum: `RequestInit::headers` / `ResponseInit::headers` → `Option<HeadersKind>` (`Headers` / `Object<String>` / `JsValue` for `string[][]`), `RequestInit::body` → `Option<BodyKind>`; unions of undiscriminable dictionaries erase instead: `TraceItem::event` (`EventKind`, 8 arms) → `Option<JsValue>`, `CryptoKey::algorithm` (`AlgorithmKind`) → `JsValue`, tail-stream `Onset::info` / `SpanOpen::info` | main #187 / #192 (alias enums), #184 (`is_object` checks: arms that are all plain dictionaries cannot be told apart, so the old enum's `from_js` was never sound) | `response.rs` `headers_of` matches `HeadersKind` | improvement for `headers` / `body`; **regression** for tail workers (`event` loses its enum, though it was undiscriminable) |
| 4 | type parameters filling a `Function<fn(..)>` slot bounded `MonoGeneric` (65: `ReadableStream::new<R: MonoGeneric>`, `WritableStream::new<W>`, `Cache::*<CfHostMetadata>`, `R2Bucket::put<R>`), nested ones `'static` (50); inline `RefIntoWasmAbi` / `FromWasmAbi` bounds moved to `where` clauses | js-sys's `Function` slots are `MonoGeneric` since wasm-bindgen#5390; `state_abi_bounds` | none | neutral; without it the bindings had 608 errors |
| 5 | awaited generic `Option<T>` reads bounded `Option<T>: FromWasmAbi` (10: `KVNamespaceListKey::metadata<Metadata>`, `Request::cf<Cf>`) | ts-gen "Bound a generic Option<T> read whole": `f64` / `i64` are not `OptionFromWasmAbi` | none (`get_with_metadata::<f64>` compiles again) | improvement |
| 6 | `Promise<T, crate::Error>` rejections cross as `E` | #5426 as posted (`E: FromWasmAbi + 'static`, not `From<JsValue>`) | `error.rs`: `WasmDescribe` + `FromWasmAbi` for `Error` through `JsValue` | neutral; glue a `Promise<T, E>` consumer must write once |
| 7 | `KVNamespaceGetOptionsPartial<Type>` → `PartialKvNamespaceGetOptionsUndefined` (non-generic, `cache_ttl(f64)`); the typed read wrappers take `&KVNamespaceGetOptions<String>` | main #177 evaluates `Partial<..>` itself; the prototype's synthesis dropped | `kv.rs` `get_options()` builds a `KVNamespaceGetOptions<String>` with a placeholder `type` the wrapper overwrites | **regression**: a required `type` the caller never chose |
| 8 | `number \| bigint` arguments `impl JsBigIntOrNumberLike` → `f64` + `_with_big_int` arm (15: `FixedLengthStream::new`, `high_water_mark`, `expected_length`, `set_inactivity_timeout`) | main's union lowering; the prototype's `JsBigIntOrNumberLike` is not on the stack | `r2.rs` test `FixedLengthStream::new(256.0)` | **regression**: integer literals no longer pass |
| 9 | `Exclude<CertVerificationStatus, "NONE">` → `ExcludeCertVerificationStatusNone`; `D1Result<JsValue>` / `KVNamespaceListResult<JsValue>` written with defaults elided; `derive_bits_with_null`, `Response::new_with_null`, `set_attribute_with_{f64,str}` arms | main #177 naming; main's `_with_null` arms for a written `null` | none | neutral |
| 10 | optional-only setters take the value bare (`set_content_id(impl JsStringLike)`, `UnderlyingSource::set_type`) | main: omission is not a write | none | neutral |
| 11 | `Map<String, Nullable<String>>` cannot be iterated by value (`V: MonoGeneric`; `&Nullable<T>` has no `IntoWasmAbi`) | wasm-bindgen residual `Map` iteration bound | `kv.rs` `collect` walks the map as `JsValue`s | **regression** (wasm-bindgen) |

| 12 | arm naming follows main #191: the enum arm is primary and the scalar gets the suffix (`country(CountryKind)` + `country_with_iso3166_alpha2_code`, `quality(QualityKind)` + `quality_with_f64`, `repeat` / `stdin` likewise); `delete_with_key_and_options` → `delete_with_options`, `put_with_key_and_value_and_options` → `put_with_options`, `add_event_listener_*_with_bool` → `_with_function_and_bool`, `new_with_str` → `new_with_str_and_str` (URL), `set_web_socket_auto_response_with_req_resp` → `_with_maybe_req_resp` | main #191 "name a variant for what it adds to the bare form" | `auto_response.rs` test rename | neutral; `_with_function_` and `_with_maybe_` are noise |
| 13 | `impl Default` on 57 dictionaries; `Display` + `try_to_string` for `URL` / `URLSearchParams` | main #190, #185 | none | improvement |
| 14 | `edge_request_keep_alive_status` → `EdgeRequestKeepAliveStatusKind` (was `f64`) | main #192 | none | improvement |

### Migration-table deltas (for workers-rs-4 / MIGRATING.md)

- Every empty-admitting read is an `Option<T>` (`kv.get_text(k)`, `response.body()`, `r2.get(k)`,
  `init.fetcher()`); type arguments keep `Nullable` (`Map<String, Nullable<String>>` from the bulk
  KV reads). A `?: any` read (`response.cf()`) is a bare `JsValue`, `undefined` or `null` in-band.
- `init.headers()` is an `Option<HeadersKind>` (`Headers`, `Object<String>`, or the `string[][]`
  `JsValue`); `new Headers(init)` normalises any of them.
- `number | bigint` parameters take `f64` (`FixedLengthStream::new(256.0)`), with a
  `_with_big_int` arm.
- Generic constructors over a `Function` slot need `MonoGeneric` element types
  (`ReadableStream::<Uint8Array>::new()`); `JsValue` qualifies.

