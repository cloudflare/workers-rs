# Error model

Goal: `ts-gen-worker` handlers read `fn handler() -> Result<Response>` with `?`
on every binding call, as `worker` users write today, and a caught JS exception
is matchable by class (`Error::Js(JsException::TypeError(_))`) rather than a bare
`JsValue`.

## Done

ts-gen, branch `agent/errors` (published to `../../workers-rs/ts-gen`), one PR
per commit, each independent:

- `ad88fe7` `--error-type <path>`: configurable default error type for
  fallible bindings; `--errors-as-error` is now sugar for
  `--error-type ::js_sys::Error`. `CodegenContext::from_module_full` takes
  `&GenerateOptions` and validates the path.
- `0cb61de` `--fallible`: sync methods / static methods / free functions /
  constructors emit one `catch` binding under the primary name; `@throws {never}`
  makes them plain; getters/setters stay infallible except a getter annotated
  `@throws {X}` emits `getter, catch` returning `Result<T, X>`. `GetterMember`
  and `StaticGetterMember` carry `Throws`.
- `98532d0` `--error-enum [NAME]`: every `@throws` type across the output
  becomes a variant of one generated enum (default `JsException`) with
  `From<JsValue>` dispatching most derived class first, `Into<JsValue>`,
  `AsRef<JsValue>`, `Display`, `std::error::Error`. All `@throws`-typed
  bindings return it; untyped keep the `--error-type` default. Unknown names
  are an error diagnostic.

workers-rs, branch `gbedford/tgw-errors`:

- `db76e80` wires it in: `src/error.rs` (`Error { Js(JsException), Rust(String) }`
  + `Serde` behind `serde`, `Result<T>`, `JsValue` conversions), chomp task
  flags `--fallible --error-type crate::Error --error-enum JsException`, `@throws`
  in `types/worker.d.ts`, test worker on `Result<Response>` + `?`, three new
  miniflare tests (TypeError match, RangeError match, `From` + `Display`),
  submodule pointer, regenerated bindings. 18/18 miniflare tests, fmt / check
  / clippy clean.

## Decisions

- `--errors-as-error` emits `::js_sys::Error` (fully qualified) instead of the
  previous bare `Error`. Bare `Error` relied on the `use js_sys::*` glob and
  would rebind if the input declared its own `Error` class (as `workers-types`
  does). Snapshot and `worker/src/bindings/email.rs` change only in spelling.
- `--error-type` is parsed with `syn::parse_str::<syn::Type>` and emitted
  verbatim, so callers pass `crate::…` or `::some_crate::…`. The test crate's
  `Env` is generated with `--error-type ts_gen_worker::Error` and no
  `--error-enum`, so app `Env` bindings share the crate's `Error` rather than
  growing a second enum.
- Getter `@throws {X}` is honored under both policies, not only `--fallible`.
  There is no `try_` form for getters, so an explicit annotation has one
  possible lowering; gating it on the flag would make the default policy
  silently drop a declared failure mode. No existing fixture had `@throws` on
  a property, so no default-policy snapshot moved.
- With `--error-enum` on, a single-type `@throws {TypeError}` routes through
  the enum too (as the plan proposed). Trade-off: the static signature no
  longer says which class a call throws (the rendered `## Errors` doc still
  does), in exchange for one `?`-able type and `From<JsException> for Error`
  being the only conversion a project needs.
- The enum is named `JsException`, not the plan's `JsError`. The idiomatic
  handler file does `use ts_gen_worker::*; use wasm_bindgen::prelude::*;`,
  and `wasm_bindgen::prelude::JsError` makes `JsError` an ambiguous glob
  import (E0659) the first time a migrating user matches on it. The ts-gen
  CLI default and the chomp flag both use `JsException`; the test worker
  names it through the glob alone.
- The enum is emitted whenever the flag is given, even with zero `@throws`,
  so `error.rs` can reference `crate::JsException` unconditionally.
- `JsException::from` dispatch order is a topological order over ancestor
  chains (a class before every ancestor), deeper chains first, source order
  for ties. A class whose ancestry ts-gen cannot see (`--external` mapping,
  no `extends` in the input) is placed before every builtin-rooted class:
  plain depth sorting tied it with `Error` and let the `Error` arm swallow
  it. Found in review; `worker.d.ts` has no externals so its bindings are
  unchanged.
- `Display` of `JsException` is the inner `message` for an `Error` instance;
  for a non-`Error` throwable it is the string itself, else `toString()`,
  else `Debug`. Never panics.
- `ts_gen_worker::Error` also has `From<String>` / `From<&str>` → `Rust`,
  mirroring `worker::Error`. `From<Error> for JsValue` turns `Rust`/`Serde`
  into a `js_sys::Error` with the message so a handler can return either.
- `@throws` in `worker.d.ts` is limited to classes the Fetch spec pins:
  `TypeError` on `Headers` name/value validation (constructor, `append`,
  `delete`, `get`, `has`, `set`), `Request`/`Response` constructors and
  `clone`, `Response.json`, `ReadableStream.getReader`; `RangeError |
  TypeError` on `Response.redirect`. `fetch`, `text`, `arrayBuffer`,
  `cancel`, and `ExecutionContext` stay untyped (`crate::Error`). `console`
  functions get `@throws {never}` so `console::log(..)` stays `()`.
- The miniflare spec's `type: "worker"` in the worker config was removed:
  the locked `miniflare@5.20260923.0-alpha` (what `npm ci` installs in CI)
  rejects it; the older `5.20260910.0-alpha` in the master worktree's
  `node_modules` required it. Not an error-model change, but the suite could
  not run without it.

## Open problems

- **Policy for `@throws` beyond the Fetch spec.** Only spec-pinned classes
  were annotated. Whether to annotate runtime-specific throws in
  workers-types (e.g. KV/R2/D1 errors, `DOMException` names) needs a
  position on how authoritative `@throws` must be, since a wrong annotation
  still lands the value in `Other`, never a wrong variant — so over-annotating
  is safe but misleading in docs.

- **`@throws {never}` on a constructor is ignored.** Constructors always
  catch (pre-existing rule, kept under `--fallible`), so a `.d.ts` cannot
  declare an infallible constructor and `Headers::new()` stays `Result`
  even when annotated. Proposal: honor `@throws {never}` on constructors as
  on methods (wasm-bindgen accepts a non-`catch` constructor), or emit a
  warning diagnostic that the annotation has no effect. Low priority; no
  constructor in `worker.d.ts` is annotated `never`.
- **`Display` for a thrown non-string primitive is `Debug`** (`JsValue(42)`,
  `JsValue(null)`): `toString()` is only called on objects because it throws
  on `null`/`undefined`. Acceptable for an edge case; a per-primitive
  (`as_f64`, `as_bool`, `is_null`) rendering would be the fix if it matters.

## Gaps found

- ts-gen: declarations inside a `declare namespace` are emitted in a
  `pub mod ns { use wasm_bindgen::prelude::*; … }` that does not import the
  parent module's types (`Foo`, `TypeError`, …), so a namespaced function
  referencing a global type does not compile. `JsException` inside a namespace
  would hit the same (`console` here is fine: `@throws {never}` and no
  global types). Pre-existing; not touched.
- ts-gen: TS `URIError` is not mapped to `js_sys::UriError`
  (`JS_SYS_RESERVED` lists `UriError`, `BUILTIN_PARENTS` lists `URIError`),
  so `@throws {URIError}` is "unresolved". Pre-existing.
- ts-gen: `DOMException` is not in `WEB_SYS_DEFAULTS`; an input must declare
  it (workers-types does) or pass `--external DOMException=::web_sys::DomException`.
- ts-gen-worker: `HeadersInit` / `RequestInfo` unions still erase to `JsValue`
  in `RequestInit`/`ResponseInit` setters (pre-existing warning on regen).

## Session 2: ts-gen fixes from the test-suite port

ts-gen `agent/errors` reset to `gbedford/workers-parity` (`a643100`), three new
commits, published to `../../workers-rs/ts-gen`:

- `aa5d2b3` inner-position nullable lowering. `TypeRef::Nullable` now carries
  `ir::Nullish` (`Null` when the source names `null`, `Undefined` when only
  `undefined`/`void`). Inner positions lower `Null` to `JsNullable<T>` and
  `Undefined` to `JsOption<T>`; outer positions unchanged. Fixture
  `nullable-inner`.
- `67fa212` `null` arguments. A parameter naming `null` keeps a
  `_with_null(&Null)` variant unless it is optional and nothing follows it in
  that variant (a variadic tail counts as following). Only `Nullish::Null`
  qualifies; `any | null` is already `&JsValue`. Fixture `nullable-args`.
- `4a1327a` `<name>_promise`. Every `Promise`-returning callable also emits a
  sync sibling returning `Promise<T>`, named after the primary
  (`fetch_with_init_promise`). Plain under the default policy;
  `Result<Promise<T>, E>` under `--fallible`, `@throws` typed, `@throws
  {never}` plain. Fixtures `promise-companion`, `promise-companion-fallible`.

workers-rs `gbedford/tgw-errors` (rebased onto `gbedford/ts-gen-worker`
`3fec3ab`): `0f776cf` bumps the submodule, regenerates `bindings.rs` and
`worker/src/bindings/email.rs`, removes `util::nullable`, the extra
`constructor(body: null, …)` overload and the R2 empty-string stand-in for
`put(key, null)`, and routes `util::block_concurrency_while` through the
`_promise` form so the DO constructor gate is entered synchronously. 23 spec
files, 176 tests green; fmt / check / clippy clean.

### Decisions

- **IR distinction instead of "everything → `JsNullable`".** The plan allowed
  lowering all three spellings to one nullish wrapper. That is correct on the
  read side but PR 2 needs the write side: a `_with_null` variant for a
  parameter declared `T | undefined` passes `null` where the API said
  `undefined` (`new File(bits: … | undefined, name)` in workers-types would have
  grown `new_with_null`). The parser's coalescing was the common cause, so
  `Nullable` now records whether `null` was written. Cost: `Nullable(inner)`
  became `Nullable(inner, _)` at ~40 match sites in `typemap.rs` /
  `signatures.rs` / `ir.rs` / parse; workers-rs-3's `agent/workers-types` adds
  ~10 more `Nullable` sites and will need the same one-token edit on rebase.
  Benefit beyond correctness: `T | undefined` keeps wasm-bindgen's precise
  `JsOption<T>` (DO storage `get`, `Cache.match`) rather than widening.
- **Null-arm rule is "required, or followed by something", not the plan's
  "followed by further parameters, or sole constructor parameter".** The
  constructor case in the plan was `Response`, whose `body?` is optional:
  `new Response(null)` is `new Response()`, so the arm is dropped there and
  `new_with_null_and_init` is what survives. A required trailing `T | null`
  (`clearTimeout(null)`, R2 `put(key, null)`) is only reachable through
  `_with_null`, so it is kept. Setters follow the same rule (their one param is
  required): workers-types gains ~140 `set_x_with_null` / builder
  `x_with_null` methods, almost all on the AI model dictionaries whose fields
  are `x?: T | null`. Union setters already fan out, so this is consistent;
  if the noise is unwanted, skipping the arm for `SignatureKind::Setter` is a
  one-line change in `expand_single_overload`'s caller.
- **Companion, not `Promise<T>` primary.** `Promise<T>: IntoFuture` exists in
  js-sys but its output is `Result<T, JsValue>`, which would discard
  `--error-type` and `@throws` on every async binding and, under `--fallible`,
  turn a sync throw plus rejection into `?.await?`. The additive sibling keeps
  the awaited form as the ergonomic default. Emitted unconditionally (no flag),
  like the `try_` companion: it doubles the async surface, including
  `worker/src/bindings/email.rs`, but every form is a real JS call.
- **`_promise` catches under `--fallible`.** The plan said "`catch`-free since
  the rejection is the promise's". That holds for JS `async` functions, but
  workerd's promise-returning methods are JSG wrappers whose argument type
  checks and `JSG_REQUIRE`s throw synchronously, so under a policy whose
  contract is "no
  binding aborts" the sync form catches too: `Result<Promise<T>, E>`, one `?`
  before handing the promise to `waitUntil`. Under the default policy it is
  plain like every other sync primary.
- Suffix placement: `_promise` goes after the `_with_X` disambiguator
  (`fetch_with_init_promise`), following `_js_string`, not before it as the
  `try_` prefix does.

### Open problems

- **`Cache.match` resolves `undefined`, not `null`.** The plan listed it with
  KV `get` / `getAlarm`; workers-types declares `Promise<Response | undefined>`
  and the Cache API spec agrees, so it stays `JsOption<Response>` and the port
  already used `into_option()` there. Nothing to change, noted so the plan's
  list is not taken as a bug.
- **`Nullish` rebase cost for workers-rs-3** (see Decisions): every
  `TypeRef::Nullable(inner)` pattern on `agent/workers-types` needs `, _`, and
  constructions need a `Nullish` value (`Undefined` for `?:` widening, `Null`
  for a written `null`). Mechanical, compiler-guided.
- **`_promise` for a `Promise`-typed property** is not emitted (the getter is
  already sync). A property typed `() => Promise<T>` is a `Function` and also
  gets nothing. Both are fine; recorded as the boundary of the rule.

### Gaps found

- ts-gen: `type_snake_name` gives `Nullable(inner, _)` the inner's name, so a
  union param `string | Foo | null` suffixes as `_with_str` / `_with_foo` /
  `_with_null`; fine, but `putWith` + `_with_null` reads as
  `put_with_with_null`. Pre-existing suffix mechanics.
- ts-gen-worker: `util::block_concurrency_while` is the shape
  `src/ext/durable.rs` should expose (workers-rs-4): a Rust future run inside
  the gate, entered synchronously via the `_promise` form.

## Session 3: small ts-gen fixes from the "Gaps found" lists

ts-gen `agent/errors`: six commits, rebased onto `gbedford/workers-parity`
`1063861` (workers-rs-3's 23 commits) and published to `../../workers-rs/ts-gen`.
Two listed items were already on that branch and were skipped: overload
base-name ownership (`f3f093a`) and the namespace `pub mod` preamble
(`8492113`).

- `ea68276` `URIError` / `DOMException`. `URIError` resolves through a new
  `external_map::JS_SYS_RENAMES` table (`::js_sys::UriError`, kept under
  `--no-web-sys`); `DOMException` joins `WEB_SYS_DEFAULTS`
  (`::web_sys::DomException`). The builtin scope now iterates those tables
  instead of a duplicate name list. Fixture `builtin-errors`.
- `0a80171` `declare const` statics render `SCREAMING_SNAKE_CASE`
  (`pub static CACHES`, `js_name = "caches"`). Fixture coverage in `basic`,
  `coverage`, `es-module-lexer`.
- `f70faa4` instance members own their base name: a static whose base an
  instance member owns takes `_static` (`Response::json_static`,
  `json_static_with_init`, `ok_static`); a synthesized `try_` that would spell
  a declared method yields to it. Fixture `static-collision`.
- `9d90158` awaited values lower like sync returns: `String` / `f64` / `bool`
  / `()` / `Option<T>` / `Vec<T>` / synthesized return enums; the `_promise`
  form keeps the JS-typed `Promise<JsString>` etc. An `Option` / `Vec` over an
  in-scope type parameter keeps `JsOption<T>` / `Array<T>` (see Decisions).
  Removes the per-mono `mono_native_return` special case. Fixtures updated
  across the board.
- `ec92ad2` inline method return literals hoist like parameters
  (`SqlStorageCursorNextReturn<T>` with `done()` / `value()`); `never` →
  `Undefined`; union construction keeps one member per distinct lowering, so
  `{…} | {…}` properties are `Option<Object>`. Fixture `inline-return`.
- `104a370` a callback type with a rest parameter lowers to the untyped
  `Function`. Fixture `callback-types`. New CONVENTIONS section "Function
  types" (there was none).

workers-rs `gbedford/tgw-errors` (on `gbedford/ts-gen-worker` `c44c7b4`, the
real workers-types input): the submodule bump regenerates `bindings.rs` (email
bindings unchanged) and drops the port's workarounds: `util::text` /
`util::response_text`, every `String::from(x.text().await?)`, `.value_of()`,
`.into_option()` on async results, the `Reflect::get` reading of
`SqlStorageCursor::next()`, `resolve.unchecked_ref()` for `setTimeout`, plus
`caches` / `console` → `CACHES` / `CONSOLE` and the crate's
`non_upper_case_globals` allow. With the real input `Response.json` no longer
collides (the instance `json()` is on `Body`), so `_static` is not triggered
in these bindings. Both chomp test tasks green; fmt / check / clippy clean.

### Decisions

- **Statics are upper-case rather than carrying an `#[allow]`.** The item
  cannot: wasm-bindgen's `ImportStatic` keeps no attributes, so anything on
  the foreign static is dropped at expansion (verified in
  `crates/macro-support/src/codegen.rs`), and an inner `#![allow]` would not
  survive `include!`. Upper-casing is the same rule that snake-cases members.
  Consumers rename `caches` → `CACHES`; the crate-level
  `non_upper_case_globals` allow in `ts-gen-worker/src/lib.rs` is dropped.
- **`_static` on the static, not a suffix on the instance.** `r.json()` is
  the overwhelmingly common call and MDN already disambiguates the same clash
  as `Response/json_static`. Instance names are *reserved* before emission
  rather than emitted first, so output stays in declaration order. Side
  effect: a declared `try_count()` now beats `count()`'s synthesized
  `try_count` companion (`phase1` snapshot), which is the better owner.
- **Async returns use the sync lowering outright, not a primitive-only
  carve-out.** The premise behind the wrappers (`Promise<T>` needs
  `T: JsGeneric`) does not apply to the awaited value: `JsFuture<T>` resolves
  through a `Closure::once(|val: T|)` needing only `FromWasmAbi`. Verified at
  runtime against the fork's CLI with `String`, `f64`, `bool`, `()`,
  `Option<String>` (`null` and `undefined` both `None`), `Option<f64>`,
  `Option<bool>`, `Option<Object>`, `Vec<String>`, `Vec<f64>`, `Vec<Object>`,
  and rejection → `Err`. A mismatched JS value (number for `Promise<string>`)
  aborts in the shim, exactly as a sync `-> String` import does, so the
  robustness trade is unchanged. One carve-out surfaced on the real input:
  `DurableObjectStorage.get<T>(): Promise<T | undefined>` cannot be
  `Option<T>` — the macro requires `<Option<T> as FromWasmAbi>` for the
  awaited value and `T: JsGeneric` does not imply `OptionFromWasmAbi`
  (a sync import erases `T` to its concrete representation first, which is
  why `SyncKvStorage::get<T>() -> Option<T>` is fine). So an `Option` / `Vec`
  whose element mentions an in-scope type parameter keeps the inner lowering
  (`JsOption<T>` / `JsNullable<T>` / `Array<T>`); `util::storage_get` keeps its
  `into_option()`. This supersedes Session 2's
  `Promise<T | null>` → `JsNullable<T>` on the awaited side: `Option<T>` is
  nullish-insensitive, so the `JsNullable` / `JsOption` distinction only
  matters for the `_promise` form and other inner positions now.
- **The `_promise` form keeps JS-typed parameters.** `Promise<String>`
  compiles and awaits, but the promise object is handed to JS APIs
  (`waitUntil`, `Promise.race`) that need `T: JsGeneric` to upcast to
  `Promise<JsValue>`. So `get()` → `Option<String>` beside `get_promise()` →
  `Promise<JsNullable<JsString>>`.
- **Return-literal hoisting is limited to methods with a parent name**, the
  same boundary as parameter hoisting; free functions keep `Object`. Property
  literals are not hoisted either (191 in workers-types; the documented
  "hoisting scope" rule), which is why the duplicate-member collapse was also
  needed: it fixes `{…} | {…}` wherever hoisting does not reach, by
  construction in `simplify_union` so every consumer (LUB, dynamic-union
  synthesis) sees distinct members.
- **`never` → `Undefined`, not a new `TypeRef` variant.** A `never`-typed slot
  can only read back `undefined`; mapping it there lets the existing
  `undefined` coalescing turn `value: T` / `value?: never` into `Option<T>`
  without touching ~40 `TypeRef` match sites. Snapshot impact beyond the
  merge: `declare var onmessage: never` is `Undefined` instead of `JsValue`.
- **Variadic callbacks → untyped `Function`**, not `Function<fn(JsValue…)>`:
  a `fn` pointer has no variadic spelling, and `(...args: any[]) => void` is
  "any function" in intent. The typed form is still emitted for fixed-arity
  callbacks.

### Open problems

- **Shared dynamic-union names.** Identity is the ordered member list, so
  `AIGatewayHeaders.cf-aig-custom-cost: {…} | {…} | string` now renders as
  `AudioKind` (the first `Object | String` anchor in workers-types). Pre-existing
  rule, now more visible because duplicate members collapse; a per-anchor
  name (or `ObjectOrStringKind`-style structural name) would read better.
- **`never` properties.** `D1Response.error?: never` is now
  `Option<Undefined>` with a `set_error(&Undefined)` setter and builder
  method; harmless but noise. Skipping setters for `never`-typed properties
  would be a one-line rule in the property converter.

### Gaps found

- ts-gen: `() => void` callbacks render `Function<fn() -> Undefined>`; a
  `Closure<dyn FnMut()>` still needs `unchecked_ref` into that exact type. A
  `Function<fn()>`-shaped lowering for void returns, or accepting `&Function`
  for any typed callback parameter, would remove the last `unchecked_ref`s.
- ts-gen: integration tests have no async-primitive coverage; the runtime
  check for `91e0bf4` was done against the fork's CLI in a scratch crate.
  Worth a `tests/*.rs` case under `integration-tests/` once that harness
  runs with the fork's `wasm-bindgen-test-runner`.

## Session 4: the three skipped specs

workers-rs `gbedford/tgw-errors` on `gbedford/ts-gen-worker` `cb8cd7d`; no
ts-gen change. No spec file is `describe.skip` any more: `signal` and
`container` are ported, `panic` passes on the module target with a
class wrapper in the test harness. 26 spec files, 191 tests, 187 pass and 4
skip (upstream's own `subrequest` / `socket` skips plus the two `container`
tests when `TEST_CONTAINER_NAME` is unset); same under `test:ts-gen-worker-http`.

- `d83763e` `signal`: `worker/src/signal.rs` ported verbatim as
  `ts_gen_worker::signal`, `/signal/poll` route, spec unskipped.
- `706c175` `container`: `EchoContainer` on the generated
  `DurableObjectState::container()` / `Container::{running,start,get_tcp_port}`
  / `Fetcher::fetch`, websocket relay over the test-local event channel.
- `2211cb8` panic recovery: `src/init.rs` registers `set_on_abort(schedule_reinit)`,
  the chomp build passes `--force-enable-abort-handler`, `manifest.ts`
  generates the Durable Object reinit wrapper, `panic.spec.ts` unskipped
  (verbatim upstream).

### `signal`: the runtime, not the shim

`is_registered()` was `true` on the first run with no harness change.
workerd patches `WebAssembly.instantiate` and `new WebAssembly.Instance`
(`checkAndRegisterShutdown` in `src/workerd/io/wasm-instantiate-shim.js`,
contract in `io/tracked-wasm-instance.h`) to look for exported
globals named `__instance_signal` and `__instance_terminated` and register
their addresses with the C++ side. A `#[no_mangle] static` becomes an
exported `global` holding its address; wasm-bindgen keeps it, and the module
target instantiates with `new WebAssembly.Instance(wasmModule, imports)`.
So worker-build is not involved; `wasm-tools print --skeleton` on the module
build shows both exports, and the spec asserts `true:false`.

### `container`: works, with two environment facts

The spec passes against Docker locally (`TEST_CONTAINER_NAME=1`), both
`post-echo` and the websocket relay. Two things were needed that the
original suite also needs and that are not in the repo:

- the `worker-dev/echocontainer:latest` image (built from
  `test/container-echo`; the Dockerfile's in-container `cargo build` fails
  behind the corporate TLS interception, so the binary was built on the host
  and copied into a `debian:bookworm-slim` image), and
- workerd's sidecar image `cloudflare/proxy-everything:3cb1195@sha256:0ef67…`.
  workerd does not pull it; wrangler normally does. Without it
  `Container.start()` fails silently (the start RPC is `sendIgnoringResult`),
  `monitor()` rejects with "Container failed to start", and every
  `getTcpPort().fetch()` throws "Container ingress proxy is not running".
  Found by proxying the Docker socket; the original suite times out
  identically on this machine until the image is pulled.

The port's readiness probe retries on any error, not on the one message the
original matched ("container port not found"): this workerd says "Container
is not listening to port 8080" while the container boots, so the original
would `throw_str` on the first probe and hang `fetch` forever. Bounded loop,
same attempt count. `Container::start()` is the generated zero-arg form
(`start_with_options` exists for the dictionary); the DO returns the container
port's `Response` directly, no `http` conversion since the port's handler type
is fixed.

### `panic`: design and what was done

What the spec needs, in order: (1) a trap in a handler surfaces as a 500 with
"Workers runtime canceled" (that text is workerd's hang detector: the trap
escapes the microtask that polls the handler future, so the fetch promise
never settles); (2) the next call into the module runs on a fresh instance;
(3) Durable Object instances the runtime still holds work again, with reset
in-memory state (`unstored_count: 1`).

The wasm-bindgen fork already carries (2) with no shim. `--force-enable-abort-handler`
wraps every export and closure call in `__wbg_call_guard` / `__wbg_handle_catch`;
a trap sets `__instance_terminated` and runs the Rust `set_on_abort` hook,
which calls `schedule_reinit()`; the next guarded call runs `__wbg_reset_state()`
(new `WebAssembly.Instance`, reset heap/memory views, `__wbindgen_start`). This
was verified first in isolation: with only `src/init.rs` and the flag, panic,
`unreachable`, a JS exception crossing Wasm frames and OOM each gave the
canceled 500 and the next plain request succeeded, while a pre-existing
Counter DO failed with the glue's own "Invalid stale object from previous
Wasm instance". That error is (3), and it is the only thing left for a shim
to do. The flag-free default build would instead keep calling into a
poisoned instance (shadow stack pointer not unwound, `RefCell` borrows held),
so the flag belongs in every ts-gen-worker build, not only the test.

**(a) worker-build target.** `shim.js` assumes `export function fetch(req,
env, ctx)`-style free exports and synthesizes `Entrypoint.prototype.fetch =
function (arg) { return exports.fetch.call(this, arg, this.env, this.ctx) }`
on a `WorkerEntrypoint` subclass; it calls `exports.__worker_init_state()`
unconditionally; it runs esbuild over the result. ts-gen-worker's `#[event]`
already emits the module-worker shape (`export { _default as default }` with
`default.fetch`), so `generate_handlers` would mis-wrap `default` and the
`start` functions and would need a pass-through mode, `__worker_init_state`
would need to become optional, and the chomp task would inherit worker-build's
wasm-pack option parsing, lockfile and esbuild download. Its
`checkReinitialize` / `handleMaybeCritical` / `error` listener duplicate what
the fork's call guard now does. Not small; not implemented.

**(b) minimal wrapper of our own.** Implemented in the harness only
(`test/tests/manifest.ts`, `reinitShim`), nothing in `src/`: re-export
`default`, wrap each `export class` in a `Proxy` whose instances catch the
glue's "Invalid stale object" error on a method call, re-run the constructor
with the saved arguments and retry the call once. The glue's own stale check
is the signal, so no instance counter is needed (an earlier version bumped a
`globalThis` counter from the crate's start function; dropped, since it put
harness support in the crate and, see the open problem below, would still
miss the case the counter was meant to catch). workerd reads `fetch` /
`alarm` / `webSocket*` off the DO instance once at construction and calls the
cached functions, so binding methods at `get` time is not enough; the
returned function is itself a `Proxy` whose `apply` resolves the current
instance. With that, `panic.spec.ts` passes verbatim. Generated from the
class names in `index.js`, same job as worker-build's `add_export_wrappers`.

**Recommendation.** (b) for now, in this shape, but it is a stand-in for a
third option that removes the shim entirely: **(c) an opt-in class attribute
in the wasm-bindgen fork** (`#[wasm_bindgen(reinit = "reconstruct")]`, set
by `#[durable_object]`) so the generated class saves its constructor
arguments and, where the method prelude currently throws "Invalid stale
object", re-runs the constructor and refreshes `__wbg_ptr` / `__wbg_inst` in
place. The glue already tracks `__wbg_instance_id` and per-object `__wbg_inst`,
so the check exists; the recovery action is new, and the check has to move
after `__wbg_call_guard()` (see the open problem below) for it to fire in
every case. That gives plain
`wasm-bindgen --target module` output panic recovery for Durable Objects with
no wrapper module, no instance counter on `globalThis`, and no build tool; the
runtime's cached methods keep working because they are the class's own
prototype methods over a refreshed `__wbg_ptr`. Reconstruction is only valid
for classes whose contract allows it (Durable Objects are re-created across
evictions anyway), hence opt-in rather than the current throw-for-all.

### Decisions

- `src/init.rs` is crate code, not test code: the abort hook is needed under
  every option and `worker` registers the same one (`worker/src/init.rs`).
  It carries nothing else; the Durable Object wrapper is harness-only and
  marked as a stand-in.
- No `criticalError` flag from the panic hook. `worker` sets it so the shim
  reinits before the trap is observed; with the fork's guard the trap itself
  is the signal, and the test worker's own hook still logs the message.
- The container readiness probe tolerates any error (see above); the
  alternative, matching this workerd's wording, would break again on the
  next change.

### Open problems

- **(c) is a wasm-bindgen fork change**, not ts-gen or workers-rs; needs
  Guy's call on attribute spelling and whether `#[durable_object]` sets it
  implicitly. Until then the test harness carries the wrapper and a user of
  the module target has to write the same wrapper (or use worker-build once
  it understands the `default` export shape).
- **Fork glue: the stale check runs before the call guard.** In
  `crates/cli-support/src/js/binding.rs` the method prelude emits
  `if (this.__wbg_inst !== __wbg_instance_id) throw ...` and only then
  `__wbg_call_guard()` (from `maybe_wrap_export_call`). When a Durable Object
  method is the *first* call into the glue after a trap (a direct stub call
  from another worker, an alarm, a websocket event; anything that does not
  pass through `default.fetch` first), the ids still match, the guard then
  reinstantiates, and the method runs on the fresh instance with the old
  `__wbg_ptr`. Reproduced: `mf.getDurableObjectNamespace("COUNTER")` stub
  `fetch`, then `/test-panic`, then the same stub `fetch` hangs until the
  test timeout, with or without the harness wrapper. `panic.spec.ts` never
  hits it because every post-trap request enters via `default.fetch`.
  Patching the generated `index.js` to call `__wbg_call_guard()` before the
  stale check makes the reproducer pass (the wrapper then sees the stale
  error and reconstructs). Proposed fork fix: emit `__wbg_call_guard();` in
  the method prelude ahead of the stale check (the guard is idempotent, so
  the existing one around the call can stay). Applies to every user of
  `--force-enable-abort-handler` with exported classes, not only this crate;
  (c) depends on it.
- **Container images are not provisioned by the suite.** CI never runs the
  container spec (no `TEST_CONTAINER_NAME`), and a local run needs both
  images present. The sidecar digest is baked into the workerd binary, so a
  `docker pull` step in the chompfile would have to track it.

### Gaps found

- workers-rs `test/src/container.rs` has the same stale readiness message
  and hangs on this workerd; the port's any-error retry is the fix if it is
  wanted upstream.
- `test/container-echo/Dockerfile` cannot build behind TLS interception
  (`cargo` inside `rust:alpine` has no corporate CA); a host-built binary
  copied into a base image works.
