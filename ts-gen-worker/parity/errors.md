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
