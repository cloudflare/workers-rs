# Error model

Goal: `ts-gen-worker` handlers read `fn handler() -> Result<Response>` with `?`
on every binding call, as `worker` users write today, and a caught JS exception
is matchable by class (`Error::Js(JsError::TypeError(_))`) rather than a bare
`JsValue`.

## Done

- ts-gen `ad88fe7` `--error-type <path>`: configurable default error type for
  fallible bindings; `--errors-as-error` is now sugar for
  `--error-type ::js_sys::Error`. `CodegenContext::from_module_full` takes
  `&GenerateOptions`.
- ts-gen `0cb61de` `--fallible`: sync methods / static methods / free functions /
  constructors emit one `catch` binding under the primary name; `@throws {never}`
  makes them plain; getters/setters stay infallible except a getter annotated
  `@throws {X}` emits `getter, catch` returning `Result<T, X>`. `GetterMember`
  and `StaticGetterMember` carry `Throws`.

## Decisions

- `--errors-as-error` emits `::js_sys::Error` (fully qualified) instead of the
  previous bare `Error`. Bare `Error` relied on the `use js_sys::*` glob and
  would silently rebind if the input declared its own `Error` class (as
  `workers-types` does). Snapshot and `worker/src/bindings/email.rs` change
  only in that spelling.
- `--error-type` path is validated with `syn::parse_str::<syn::Type>` at
  generate time and is a hard error otherwise; the path is emitted verbatim so
  callers pass `crate::…` or `::some_crate::…`.
- Getter `@throws {X}` is honored under both policies, not only `--fallible`.
  There is no `try_` form for getters, so an explicit annotation has one
  possible lowering; gating it on the flag would make the default policy
  silently drop a declared failure mode. No existing fixture has `@throws` on a
  property, so no default-policy snapshot moves.
- `--fallible` keeps the existing default off; the flag is documented in
  CONVENTIONS as the policy for `?`-style code.

## Open problems

(none yet)

## Gaps found

(none yet)
