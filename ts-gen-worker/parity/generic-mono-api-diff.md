# Generic mono: public API diff of `bindings.rs`

Baseline: the non-mono bindings from ts-gen `45a278c` (session 15a, `a4cbe97`). Mono: the same
command with `--experimental-generic-mono` on ts-gen `01ed819` (`workers-mono-test`: `45a278c`
plus #122), as emitted, before `mono-patch` (its 55 stopgap edits are listed in
`generic-mono.md`). Part 2 isolates #122: mono output of `45a278c` against mono output of
`01ed819`.

Generated with `cargo run --release --manifest-path ts-gen-worker/tools/Cargo.toml --bin api-diff
-- <old.rs> <new.rs>`. Every public item is keyed by module, owner type and name and compared as a
one-line signature; docs, bodies and wasm-bindgen marshalling attributes are not compared, derives
and `extends` are. `XBuilder` structs, aliases and enums are their own owners. One line per
changed item: `old` → `new`, `old` → removed, added → `new`.

## Part 1: non-mono → mono

281 owners touched; 1429 items changed, 15 removed, 388 added, 3611 unchanged.

### `(free functions)`

- `pub async fn fetch_with_str(input: &str) -> Result<Response, crate::Error>;` → `pub async fn fetch_with_str(input: impl ::wasm_bindgen::JsStringLike) -> Result<Response, crate::Error>;`
- `pub async fn fetch_with_str_and_init(input: &str, init: &RequestInit<RequestInitCfProperties>) -> Result<Response, crate::Error>;` → `pub async fn fetch_with_str_and_init(input: impl ::wasm_bindgen::JsStringLike, init: &RequestInit<RequestInitCfProperties>) -> Result<Response, crate::Error>;`
- `pub fn fetch_with_str_and_init_promise(input: &str, init: &RequestInit<RequestInitCfProperties>) -> Result<Promise<Response>, crate::Error>;` → `pub fn fetch_with_str_and_init_promise(input: impl ::wasm_bindgen::JsStringLike, init: &RequestInit<RequestInitCfProperties>) -> Result<Promise<Response>, crate::Error>;`
- `pub fn fetch_with_str_promise(input: &str) -> Result<Promise<Response>, crate::Error>;` → `pub fn fetch_with_str_promise(input: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Response>, crate::Error>;`

### `BasicImageTransformations`

- added → `pub fn background_js_string(this: &BasicImageTransformations) -> Option<JsString>;`
- added → `pub fn segment_js_string(this: &BasicImageTransformations) -> Option<JsString>;`
- `pub fn set_background(this: &BasicImageTransformations, val: &str);` → `pub fn set_background(this: &BasicImageTransformations, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_segment(this: &BasicImageTransformations, val: &str);` → `pub fn set_segment(this: &BasicImageTransformations, val: impl ::wasm_bindgen::JsStringLike);`

### `BasicImageTransformationsBuilder`

- `pub fn background(self, val: &str) -> Self;` → `pub fn background(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn segment(self, val: &str) -> Self;` → `pub fn segment(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `Blob`

- `pub fn slice_with_start_and_end_and_type(this: &Blob, start: f64, end: f64, r#type: &str) -> Result<Blob, crate::Error>;` → `pub fn slice_with_start_and_end_and_type(this: &Blob, start: f64, end: f64, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Blob, crate::Error>;`
- added → `pub async fn text_js_string(this: &Blob) -> Result<JsString, crate::Error>;`
- added → `pub fn type__js_string(this: &Blob) -> JsString;`

### `BlobOptions`

- `pub fn set_type(this: &BlobOptions, val: &str);` → `pub fn set_type(this: &BlobOptions, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn type__js_string(this: &BlobOptions) -> Option<JsString>;`

### `BlobOptionsBuilder`

- `pub fn type_(self, val: &str) -> Self;` → `pub fn type_(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `Body`

- `pub async fn json<T: ::wasm_bindgen::JsGeneric + ::wasm_bindgen::convert::FromWasmAbi>(this: &Body) -> Result<T, crate::Error>;` → `pub async fn json<T>(this: &Body) -> Result<T, crate::Error>;`
- `pub fn json_promise<T: ::wasm_bindgen::JsGeneric>(this: &Body) -> Result<Promise<T>, crate::Error>;` → `pub fn json_promise<T>(this: &Body) -> Result<Promise<T>, crate::Error>;`
- added → `pub async fn text_js_string(this: &Body) -> Result<JsString, crate::Error>;`

### `Cache`

- `pub async fn delete_with_str(this: &Cache, request: &str) -> Result<bool, crate::Error>;` → `pub async fn delete_with_str(this: &Cache, request: impl ::wasm_bindgen::JsStringLike) -> Result<bool, crate::Error>;`
- `pub async fn delete_with_str_and_options(this: &Cache, request: &str, options: &CacheQueryOptions) -> Result<bool, crate::Error>;` → `pub async fn delete_with_str_and_options(this: &Cache, request: impl ::wasm_bindgen::JsStringLike, options: &CacheQueryOptions) -> Result<bool, crate::Error>;`
- `pub fn delete_with_str_and_options_promise(this: &Cache, request: &str, options: &CacheQueryOptions) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn delete_with_str_and_options_promise(this: &Cache, request: impl ::wasm_bindgen::JsStringLike, options: &CacheQueryOptions) -> Result<Promise<Boolean>, crate::Error>;`
- `pub fn delete_with_str_promise(this: &Cache, request: &str) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn delete_with_str_promise(this: &Cache, request: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Boolean>, crate::Error>;`
- `pub async fn match_with_str(this: &Cache, request: &str) -> Result<Option<Response>, crate::Error>;` → `pub async fn match_with_str(this: &Cache, request: impl ::wasm_bindgen::JsStringLike) -> Result<Option<Response>, crate::Error>;`
- `pub async fn match_with_str_and_options(this: &Cache, request: &str, options: &CacheQueryOptions) -> Result<Option<Response>, crate::Error>;` → `pub async fn match_with_str_and_options(this: &Cache, request: impl ::wasm_bindgen::JsStringLike, options: &CacheQueryOptions) -> Result<Option<Response>, crate::Error>;`
- `pub fn match_with_str_and_options_promise(this: &Cache, request: &str, options: &CacheQueryOptions) -> Result<Promise<JsOption<Response>>, crate::Error>;` → `pub fn match_with_str_and_options_promise(this: &Cache, request: impl ::wasm_bindgen::JsStringLike, options: &CacheQueryOptions) -> Result<Promise<JsOption<Response>>, crate::Error>;`
- `pub fn match_with_str_promise(this: &Cache, request: &str) -> Result<Promise<JsOption<Response>>, crate::Error>;` → `pub fn match_with_str_promise(this: &Cache, request: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsOption<Response>>, crate::Error>;`
- `pub async fn put_with_str(this: &Cache, request: &str, response: &Response) -> Result<(), crate::Error>;` → `pub async fn put_with_str(this: &Cache, request: impl ::wasm_bindgen::JsStringLike, response: &Response) -> Result<(), crate::Error>;`
- `pub fn put_with_str_promise(this: &Cache, request: &str, response: &Response) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_str_promise(this: &Cache, request: impl ::wasm_bindgen::JsStringLike, response: &Response) -> Result<Promise<Undefined>, crate::Error>;`

### `CachePurgeError`

- added → `pub fn message_js_string(this: &CachePurgeError) -> JsString;`
- `pub fn new(code: f64, message: &str) -> CachePurgeError;` → `pub fn new(code: f64, message: impl ::wasm_bindgen::JsStringLike) -> CachePurgeError;`
- `pub fn set_message(this: &CachePurgeError, val: &str);` → `pub fn set_message(this: &CachePurgeError, val: impl ::wasm_bindgen::JsStringLike);`

### `CachePurgeOptions`

- added → `pub fn path_prefixes_js_string(this: &CachePurgeOptions) -> Option<Array<JsString>>;`
- `pub fn set_path_prefixes(this: &CachePurgeOptions, val: &[String]);` → `pub fn set_path_prefixes(this: &CachePurgeOptions, val: &[impl ::wasm_bindgen::JsStringLike]);`
- `pub fn set_tags(this: &CachePurgeOptions, val: &[String]);` → `pub fn set_tags(this: &CachePurgeOptions, val: &[impl ::wasm_bindgen::JsStringLike]);`
- added → `pub fn tags_js_string(this: &CachePurgeOptions) -> Option<Array<JsString>>;`

### `CachePurgeOptionsBuilder`

- `pub fn path_prefixes(self, val: &[String]) -> Self;` → `pub fn path_prefixes(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`
- `pub fn tags(self, val: &[String]) -> Self;` → `pub fn tags(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `CacheStorage`

- `pub async fn open(this: &CacheStorage, cache_name: &str) -> Result<Cache, crate::Error>;` → `pub async fn open(this: &CacheStorage, cache_name: impl ::wasm_bindgen::JsStringLike) -> Result<Cache, crate::Error>;`
- `pub fn open_promise(this: &CacheStorage, cache_name: &str) -> Result<Promise<Cache>, crate::Error>;` → `pub fn open_promise(this: &CacheStorage, cache_name: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Cache>, crate::Error>;`

### `CloseEvent`

- `pub fn new(r#type: &str) -> Result<CloseEvent, crate::Error>;` → `pub fn new(r#type: impl ::wasm_bindgen::JsStringLike) -> Result<CloseEvent, crate::Error>;`
- `pub fn new_with_initializer(r#type: &str, initializer: &CloseEventInit) -> Result<CloseEvent, crate::Error>;` → `pub fn new_with_initializer(r#type: impl ::wasm_bindgen::JsStringLike, initializer: &CloseEventInit) -> Result<CloseEvent, crate::Error>;`
- added → `pub fn reason_js_string(this: &CloseEvent) -> JsString;`

### `CloseEventInit`

- added → `pub fn reason_js_string(this: &CloseEventInit) -> Option<JsString>;`
- `pub fn set_reason(this: &CloseEventInit, val: &str);` → `pub fn set_reason(this: &CloseEventInit, val: impl ::wasm_bindgen::JsStringLike);`

### `CloseEventInitBuilder`

- `pub fn reason(self, val: &str) -> Self;` → `pub fn reason(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `CloudflareAccessContext`

- added → `pub fn aud_js_string(this: &CloudflareAccessContext) -> JsString;`

### `CloudflareAccessIdentity`

- added → `pub fn account_id_js_string(this: &CloudflareAccessIdentity) -> Option<JsString>;`
- added → `pub fn amr_js_string(this: &CloudflareAccessIdentity) -> Option<Array<JsString>>;`
- added → `pub fn email_js_string(this: &CloudflareAccessIdentity) -> Option<JsString>;`
- added → `pub fn ip_js_string(this: &CloudflareAccessIdentity) -> Option<JsString>;`
- added → `pub fn name_js_string(this: &CloudflareAccessIdentity) -> Option<JsString>;`
- `pub fn set_account_id(this: &CloudflareAccessIdentity, val: &str);` → `pub fn set_account_id(this: &CloudflareAccessIdentity, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_amr(this: &CloudflareAccessIdentity, val: &[String]);` → `pub fn set_amr(this: &CloudflareAccessIdentity, val: &[impl ::wasm_bindgen::JsStringLike]);`
- `pub fn set_email(this: &CloudflareAccessIdentity, val: &str);` → `pub fn set_email(this: &CloudflareAccessIdentity, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_ip(this: &CloudflareAccessIdentity, val: &str);` → `pub fn set_ip(this: &CloudflareAccessIdentity, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &CloudflareAccessIdentity, val: &str);` → `pub fn set_name(this: &CloudflareAccessIdentity, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_user_uuid(this: &CloudflareAccessIdentity, val: &str);` → `pub fn set_user_uuid(this: &CloudflareAccessIdentity, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn user_uuid_js_string(this: &CloudflareAccessIdentity) -> Option<JsString>;`

### `CloudflareAccessIdentityBuilder`

- `pub fn account_id(self, val: &str) -> Self;` → `pub fn account_id(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn amr(self, val: &[String]) -> Self;` → `pub fn amr(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`
- `pub fn email(self, val: &str) -> Self;` → `pub fn email(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn ip(self, val: &str) -> Self;` → `pub fn ip(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn name(self, val: &str) -> Self;` → `pub fn name(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn user_uuid(self, val: &str) -> Self;` → `pub fn user_uuid(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `Console`

- `pub fn count_reset_with_label(this: &Console, label: &str) -> Result<(), crate::Error>;` → `pub fn count_reset_with_label(this: &Console, label: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn count_with_label(this: &Console, label: &str) -> Result<(), crate::Error>;` → `pub fn count_with_label(this: &Console, label: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn table_with_tabular_data_and_properties(this: &Console, tabular_data: &JsValue, properties: &[String]) -> Result<(), crate::Error>;` → `pub fn table_with_tabular_data_and_properties(this: &Console, tabular_data: &JsValue, properties: &[impl ::wasm_bindgen::JsStringLike]) -> Result<(), crate::Error>;`
- `pub fn time_end_with_label(this: &Console, label: &str) -> Result<(), crate::Error>;` → `pub fn time_end_with_label(this: &Console, label: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn time_log_with_label(this: &Console, label: &str, data: &[JsValue]) -> Result<(), crate::Error>; [variadic]` → `pub fn time_log_with_label(this: &Console, label: impl ::wasm_bindgen::JsStringLike, data: &[JsValue]) -> Result<(), crate::Error>; [variadic]`
- `pub fn time_stamp_with_label(this: &Console, label: &str) -> Result<(), crate::Error>;` → `pub fn time_stamp_with_label(this: &Console, label: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn time_with_label(this: &Console, label: &str) -> Result<(), crate::Error>;` → `pub fn time_with_label(this: &Console, label: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`

### `Container`

- `pub async fn exec(this: &Container, cmd: &[String]) -> Result<ExecProcess, crate::Error>;` → `pub async fn exec(this: &Container, cmd: &[impl ::wasm_bindgen::JsStringLike]) -> Result<ExecProcess, crate::Error>;`
- `pub fn exec_promise(this: &Container, cmd: &[String]) -> Result<Promise<ExecProcess>, crate::Error>;` → `pub fn exec_promise(this: &Container, cmd: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<ExecProcess>, crate::Error>;`
- `pub async fn exec_with_options(this: &Container, cmd: &[String], options: &ContainerExecOptions) -> Result<ExecProcess, crate::Error>;` → `pub async fn exec_with_options(this: &Container, cmd: &[impl ::wasm_bindgen::JsStringLike], options: &ContainerExecOptions) -> Result<ExecProcess, crate::Error>;`
- `pub fn exec_with_options_promise(this: &Container, cmd: &[String], options: &ContainerExecOptions) -> Result<Promise<ExecProcess>, crate::Error>;` → `pub fn exec_with_options_promise(this: &Container, cmd: &[impl ::wasm_bindgen::JsStringLike], options: &ContainerExecOptions) -> Result<Promise<ExecProcess>, crate::Error>;`
- `pub async fn intercept_outbound_http(this: &Container, addr: &str, binding: &Fetcher) -> Result<(), crate::Error>;` → `pub async fn intercept_outbound_http(this: &Container, addr: impl ::wasm_bindgen::JsStringLike, binding: &Fetcher) -> Result<(), crate::Error>;`
- `pub fn intercept_outbound_http_promise(this: &Container, addr: &str, binding: &Fetcher) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn intercept_outbound_http_promise(this: &Container, addr: impl ::wasm_bindgen::JsStringLike, binding: &Fetcher) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn intercept_outbound_https(this: &Container, addr: &str, binding: &Fetcher) -> Result<(), crate::Error>;` → `pub async fn intercept_outbound_https(this: &Container, addr: impl ::wasm_bindgen::JsStringLike, binding: &Fetcher) -> Result<(), crate::Error>;`
- `pub fn intercept_outbound_https_promise(this: &Container, addr: &str, binding: &Fetcher) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn intercept_outbound_https_promise(this: &Container, addr: impl ::wasm_bindgen::JsStringLike, binding: &Fetcher) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn set_inactivity_timeout(this: &Container, duration_ms: f64) -> Result<(), crate::Error>;` → `pub async fn set_inactivity_timeout(this: &Container, duration_ms: impl ::js_sys::JsBigIntOrNumberLike) -> Result<(), crate::Error>;`
- `pub fn set_inactivity_timeout_promise(this: &Container, duration_ms: f64) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn set_inactivity_timeout_promise(this: &Container, duration_ms: impl ::js_sys::JsBigIntOrNumberLike) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn set_inactivity_timeout_with_big_int(this: &Container, duration_ms: &BigInt) -> Result<(), crate::Error>;` → removed
- `pub fn set_inactivity_timeout_with_big_int_promise(this: &Container, duration_ms: &BigInt) -> Result<Promise<Undefined>, crate::Error>;` → removed

### `ContainerDirectorySnapshot`

- `pub fn builder(id: &str, size: f64, dir: &str) -> ContainerDirectorySnapshotBuilder;` → `pub fn builder(id: impl ::wasm_bindgen::JsStringLike, size: f64, dir: impl ::wasm_bindgen::JsStringLike) -> ContainerDirectorySnapshotBuilder;`
- added → `pub fn dir_js_string(this: &ContainerDirectorySnapshot) -> JsString;`
- added → `pub fn id_js_string(this: &ContainerDirectorySnapshot) -> JsString;`
- added → `pub fn name_js_string(this: &ContainerDirectorySnapshot) -> Option<JsString>;`
- `pub fn new(id: &str, size: f64, dir: &str) -> ContainerDirectorySnapshot;` → `pub fn new(id: impl ::wasm_bindgen::JsStringLike, size: f64, dir: impl ::wasm_bindgen::JsStringLike) -> ContainerDirectorySnapshot;`
- `pub fn set_dir(this: &ContainerDirectorySnapshot, val: &str);` → `pub fn set_dir(this: &ContainerDirectorySnapshot, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_id(this: &ContainerDirectorySnapshot, val: &str);` → `pub fn set_id(this: &ContainerDirectorySnapshot, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &ContainerDirectorySnapshot, val: &str);` → `pub fn set_name(this: &ContainerDirectorySnapshot, val: impl ::wasm_bindgen::JsStringLike);`

### `ContainerDirectorySnapshotBuilder`

- `pub fn name(self, val: &str) -> Self;` → `pub fn name(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `ContainerDirectorySnapshotOptions`

- `pub fn builder(dir: &str) -> ContainerDirectorySnapshotOptionsBuilder;` → `pub fn builder(dir: impl ::wasm_bindgen::JsStringLike) -> ContainerDirectorySnapshotOptionsBuilder;`
- added → `pub fn dir_js_string(this: &ContainerDirectorySnapshotOptions) -> JsString;`
- added → `pub fn name_js_string(this: &ContainerDirectorySnapshotOptions) -> Option<JsString>;`
- `pub fn new(dir: &str) -> ContainerDirectorySnapshotOptions;` → `pub fn new(dir: impl ::wasm_bindgen::JsStringLike) -> ContainerDirectorySnapshotOptions;`
- `pub fn set_dir(this: &ContainerDirectorySnapshotOptions, val: &str);` → `pub fn set_dir(this: &ContainerDirectorySnapshotOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &ContainerDirectorySnapshotOptions, val: &str);` → `pub fn set_name(this: &ContainerDirectorySnapshotOptions, val: impl ::wasm_bindgen::JsStringLike);`

### `ContainerDirectorySnapshotOptionsBuilder`

- `pub fn name(self, val: &str) -> Self;` → `pub fn name(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `ContainerDirectorySnapshotRestoreParams`

- added → `pub fn mount_point_js_string(this: &ContainerDirectorySnapshotRestoreParams) -> Option<JsString>;`
- `pub fn set_mount_point(this: &ContainerDirectorySnapshotRestoreParams, val: &str);` → `pub fn set_mount_point(this: &ContainerDirectorySnapshotRestoreParams, val: impl ::wasm_bindgen::JsStringLike);`

### `ContainerDirectorySnapshotRestoreParamsBuilder`

- `pub fn mount_point(self, val: &str) -> Self;` → `pub fn mount_point(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `ContainerExecOptions`

- added → `pub fn cwd_js_string(this: &ContainerExecOptions) -> Option<JsString>;`
- `pub fn set_cwd(this: &ContainerExecOptions, val: &str);` → `pub fn set_cwd(this: &ContainerExecOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_user(this: &ContainerExecOptions, val: &str);` → `pub fn set_user(this: &ContainerExecOptions, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn user_js_string(this: &ContainerExecOptions) -> Option<JsString>;`

### `ContainerExecOptionsBuilder`

- `pub fn cwd(self, val: &str) -> Self;` → `pub fn cwd(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn user(self, val: &str) -> Self;` → `pub fn user(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `ContainerSnapshot`

- `pub fn builder(id: &str, size: f64) -> ContainerSnapshotBuilder;` → `pub fn builder(id: impl ::wasm_bindgen::JsStringLike, size: f64) -> ContainerSnapshotBuilder;`
- added → `pub fn id_js_string(this: &ContainerSnapshot) -> JsString;`
- added → `pub fn name_js_string(this: &ContainerSnapshot) -> Option<JsString>;`
- `pub fn new(id: &str, size: f64) -> ContainerSnapshot;` → `pub fn new(id: impl ::wasm_bindgen::JsStringLike, size: f64) -> ContainerSnapshot;`
- `pub fn set_id(this: &ContainerSnapshot, val: &str);` → `pub fn set_id(this: &ContainerSnapshot, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &ContainerSnapshot, val: &str);` → `pub fn set_name(this: &ContainerSnapshot, val: impl ::wasm_bindgen::JsStringLike);`

### `ContainerSnapshotBuilder`

- `pub fn name(self, val: &str) -> Self;` → `pub fn name(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `ContainerSnapshotOptions`

- added → `pub fn name_js_string(this: &ContainerSnapshotOptions) -> Option<JsString>;`
- `pub fn set_name(this: &ContainerSnapshotOptions, val: &str);` → `pub fn set_name(this: &ContainerSnapshotOptions, val: impl ::wasm_bindgen::JsStringLike);`

### `ContainerSnapshotOptionsBuilder`

- `pub fn name(self, val: &str) -> Self;` → `pub fn name(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `ContainerSnapshotRestoreParams`

- added → `pub fn id_js_string(this: &ContainerSnapshotRestoreParams) -> JsString;`
- `pub fn new(id: &str) -> ContainerSnapshotRestoreParams;` → `pub fn new(id: impl ::wasm_bindgen::JsStringLike) -> ContainerSnapshotRestoreParams;`
- `pub fn set_id(this: &ContainerSnapshotRestoreParams, val: &str);` → `pub fn set_id(this: &ContainerSnapshotRestoreParams, val: impl ::wasm_bindgen::JsStringLike);`

### `ContainerStartupOptions`

- added → `pub fn entrypoint_js_string(this: &ContainerStartupOptions) -> Option<Array<JsString>>;`
- `pub fn set_entrypoint(this: &ContainerStartupOptions, val: &[String]);` → `pub fn set_entrypoint(this: &ContainerStartupOptions, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `ContainerStartupOptionsBuilder`

- `pub fn entrypoint(self, val: &[String]) -> Self;` → `pub fn entrypoint(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `Crypto`

- `pub fn get_random_values<T: ::wasm_bindgen::JsGeneric>(this: &Crypto, buffer: &T) -> Result<T, crate::Error>;` → `pub fn get_random_values<T>(this: &Crypto, buffer: T) -> Result<T, crate::Error>;`
- added → `pub fn random_uuid_js_string(this: &Crypto) -> Result<JsString, crate::Error>;`

### `CryptoKey`

- added → `pub fn type__js_string(this: &CryptoKey) -> JsString;`
- added → `pub fn usages_js_string(this: &CryptoKey) -> Array<JsString>;`

### `CryptoKeyAesKeyAlgorithm`

- added → `pub fn name_js_string(this: &CryptoKeyAesKeyAlgorithm) -> JsString;`
- `pub fn new(name: &str, length: f64) -> CryptoKeyAesKeyAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike, length: f64) -> CryptoKeyAesKeyAlgorithm;`
- `pub fn set_name(this: &CryptoKeyAesKeyAlgorithm, val: &str);` → `pub fn set_name(this: &CryptoKeyAesKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `CryptoKeyArbitraryKeyAlgorithm`

- `pub fn builder(name: &str) -> CryptoKeyArbitraryKeyAlgorithmBuilder;` → `pub fn builder(name: impl ::wasm_bindgen::JsStringLike) -> CryptoKeyArbitraryKeyAlgorithmBuilder;`
- added → `pub fn name_js_string(this: &CryptoKeyArbitraryKeyAlgorithm) -> JsString;`
- added → `pub fn named_curve_js_string(this: &CryptoKeyArbitraryKeyAlgorithm) -> Option<JsString>;`
- `pub fn new(name: &str) -> CryptoKeyArbitraryKeyAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike) -> CryptoKeyArbitraryKeyAlgorithm;`
- `pub fn set_name(this: &CryptoKeyArbitraryKeyAlgorithm, val: &str);` → `pub fn set_name(this: &CryptoKeyArbitraryKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_named_curve(this: &CryptoKeyArbitraryKeyAlgorithm, val: &str);` → `pub fn set_named_curve(this: &CryptoKeyArbitraryKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `CryptoKeyArbitraryKeyAlgorithmBuilder`

- `pub fn named_curve(self, val: &str) -> Self;` → `pub fn named_curve(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `CryptoKeyEllipticKeyAlgorithm`

- added → `pub fn name_js_string(this: &CryptoKeyEllipticKeyAlgorithm) -> JsString;`
- added → `pub fn named_curve_js_string(this: &CryptoKeyEllipticKeyAlgorithm) -> JsString;`
- `pub fn new(name: &str, named_curve: &str) -> CryptoKeyEllipticKeyAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike, named_curve: impl ::wasm_bindgen::JsStringLike) -> CryptoKeyEllipticKeyAlgorithm;`
- `pub fn set_name(this: &CryptoKeyEllipticKeyAlgorithm, val: &str);` → `pub fn set_name(this: &CryptoKeyEllipticKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_named_curve(this: &CryptoKeyEllipticKeyAlgorithm, val: &str);` → `pub fn set_named_curve(this: &CryptoKeyEllipticKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `CryptoKeyHmacKeyAlgorithm`

- added → `pub fn name_js_string(this: &CryptoKeyHmacKeyAlgorithm) -> JsString;`
- `pub fn new(name: &str, hash: &CryptoKeyKeyAlgorithm, length: f64) -> CryptoKeyHmacKeyAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike, hash: &CryptoKeyKeyAlgorithm, length: f64) -> CryptoKeyHmacKeyAlgorithm;`
- `pub fn set_name(this: &CryptoKeyHmacKeyAlgorithm, val: &str);` → `pub fn set_name(this: &CryptoKeyHmacKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `CryptoKeyKeyAlgorithm`

- added → `pub fn name_js_string(this: &CryptoKeyKeyAlgorithm) -> JsString;`
- `pub fn new(name: &str) -> CryptoKeyKeyAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike) -> CryptoKeyKeyAlgorithm;`
- `pub fn set_name(this: &CryptoKeyKeyAlgorithm, val: &str);` → `pub fn set_name(this: &CryptoKeyKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `CryptoKeyRsaKeyAlgorithm`

- `pub fn builder(name: &str, modulus_length: f64, public_exponent: &ArrayBuffer) -> CryptoKeyRsaKeyAlgorithmBuilder;` → `pub fn builder(name: impl ::wasm_bindgen::JsStringLike, modulus_length: f64, public_exponent: &ArrayBuffer) -> CryptoKeyRsaKeyAlgorithmBuilder;`
- `pub fn builder_with_typed_array<T: ::js_sys::TypedArray>(name: &str, modulus_length: f64, public_exponent: &T) -> CryptoKeyRsaKeyAlgorithmBuilder;` → `pub fn builder_with_typed_array<T: ::js_sys::TypedArray>(name: impl ::wasm_bindgen::JsStringLike, modulus_length: f64, public_exponent: &T) -> CryptoKeyRsaKeyAlgorithmBuilder where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- added → `pub fn name_js_string(this: &CryptoKeyRsaKeyAlgorithm) -> JsString;`
- `pub fn new(name: &str, modulus_length: f64, public_exponent: &ArrayBuffer) -> CryptoKeyRsaKeyAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike, modulus_length: f64, public_exponent: &ArrayBuffer) -> CryptoKeyRsaKeyAlgorithm;`
- `pub fn new_with_typed_array<T: ::js_sys::TypedArray>(name: &str, modulus_length: f64, public_exponent: &T) -> CryptoKeyRsaKeyAlgorithm;` → `pub fn new_with_typed_array<T: ::js_sys::TypedArray>(name: impl ::wasm_bindgen::JsStringLike, modulus_length: f64, public_exponent: &T) -> CryptoKeyRsaKeyAlgorithm where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn set_name(this: &CryptoKeyRsaKeyAlgorithm, val: &str);` → `pub fn set_name(this: &CryptoKeyRsaKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `D1Database`

- `pub async fn exec(this: &D1Database, query: &str) -> Result<D1ExecResult, crate::Error>;` → `pub async fn exec(this: &D1Database, query: impl ::wasm_bindgen::JsStringLike) -> Result<D1ExecResult, crate::Error>;`
- `pub fn exec_promise(this: &D1Database, query: &str) -> Result<Promise<D1ExecResult>, crate::Error>;` → `pub fn exec_promise(this: &D1Database, query: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<D1ExecResult>, crate::Error>;`
- `pub fn prepare(this: &D1Database, query: &str) -> D1PreparedStatement;` → `pub fn prepare(this: &D1Database, query: impl ::wasm_bindgen::JsStringLike) -> D1PreparedStatement;`
- `pub fn with_session_with_str(this: &D1Database, constraint_or_bookmark: &str) -> D1DatabaseSession;` → `pub fn with_session_with_str(this: &D1Database, constraint_or_bookmark: impl ::wasm_bindgen::JsStringLike) -> D1DatabaseSession;`

### `D1DatabaseSession`

- added → `pub fn get_bookmark_js_string(this: &D1DatabaseSession) -> Option<JsString>;`
- `pub fn prepare(this: &D1DatabaseSession, query: &str) -> D1PreparedStatement;` → `pub fn prepare(this: &D1DatabaseSession, query: impl ::wasm_bindgen::JsStringLike) -> D1PreparedStatement;`

### `D1Meta`

- added → `pub fn served_by_colo_js_string(this: &D1Meta) -> Option<JsString>;`
- added → `pub fn served_by_region_js_string(this: &D1Meta) -> Option<JsString>;`
- `pub fn set_served_by_colo(this: &D1Meta, val: &str);` → `pub fn set_served_by_colo(this: &D1Meta, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_served_by_region(this: &D1Meta, val: &str);` → `pub fn set_served_by_region(this: &D1Meta, val: impl ::wasm_bindgen::JsStringLike);`

### `D1MetaBuilder`

- `pub fn served_by_colo(self, val: &str) -> Self;` → `pub fn served_by_colo(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn served_by_region(self, val: &str) -> Self;` → `pub fn served_by_region(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `D1PreparedStatement`

- `pub async fn first<T: ::wasm_bindgen::JsGeneric>(this: &D1PreparedStatement) -> Result<JsNullable<T>, crate::Error>;` → `pub async fn first<T>(this: &D1PreparedStatement) -> Result<JsNullable<T>, crate::Error>;`
- `pub fn first_promise<T: ::wasm_bindgen::JsGeneric>(this: &D1PreparedStatement) -> Result<Promise<JsNullable<T>>, crate::Error>;` → `pub fn first_promise<T>(this: &D1PreparedStatement) -> Result<Promise<JsNullable<T>>, crate::Error>;`
- `pub async fn first_with_col_name<T: ::wasm_bindgen::JsGeneric>(this: &D1PreparedStatement, col_name: &str) -> Result<JsNullable<T>, crate::Error>;` → `pub async fn first_with_col_name<T>(this: &D1PreparedStatement, col_name: impl ::wasm_bindgen::JsStringLike) -> Result<JsNullable<T>, crate::Error>;`
- `pub fn first_with_col_name_promise<T: ::wasm_bindgen::JsGeneric>(this: &D1PreparedStatement, col_name: &str) -> Result<Promise<JsNullable<T>>, crate::Error>;` → `pub fn first_with_col_name_promise<T>(this: &D1PreparedStatement, col_name: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<T>>, crate::Error>;`

### `D1Result`

- `pub fn results<T: ::wasm_bindgen::JsGeneric>(this: &D1Result<T>) -> Vec<T>;` → `pub fn results<T>(this: &D1Result<T>) -> Vec<T>;`
- `pub fn set_results<T: ::wasm_bindgen::JsGeneric>(this: &D1Result<T>, val: &[T]);` → `pub fn set_results<T>(this: &D1Result<T>, val: &[T]);`
- `impl<T: ::wasm_bindgen::JsGeneric> D1Result<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> D1Result<T>`
- `pub type D1Result<T: ::wasm_bindgen::JsGeneric = JsValue>; extends(D1Response, Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type D1Result<T = JsValue>; extends(D1Response, Object) derive(Debug, Clone, PartialEq, Eq)`

### `D1ResultBuilder`

- `impl<T: ::wasm_bindgen::JsGeneric> D1ResultBuilder<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> D1ResultBuilder<T>`
- `impl<T: ::wasm_bindgen::JsGeneric> From<D1Result<T>> for D1ResultBuilder<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> From<D1Result<T>> for D1ResultBuilder<T>`
- `pub struct D1ResultBuilder<T: ::wasm_bindgen::JsGeneric> { inner: D1Result<T> }` → `pub struct D1ResultBuilder<T> { inner: D1Result<T> }`

### `DigestStream`

- `pub fn new(algorithm: &str) -> Result<DigestStream, crate::Error>;` → `pub fn new(algorithm: impl ::wasm_bindgen::JsStringLike) -> Result<DigestStream, crate::Error>;`

### `DispatchNamespace`

- `pub fn get(this: &DispatchNamespace, name: &str) -> Result<Fetcher, crate::Error>;` → `pub fn get(this: &DispatchNamespace, name: impl ::wasm_bindgen::JsStringLike) -> Result<Fetcher, crate::Error>;`
- `pub fn get_with_args(this: &DispatchNamespace, name: &str, args: &DispatchNamespaceArgs) -> Result<Fetcher, crate::Error>;` → `pub fn get_with_args(this: &DispatchNamespace, name: impl ::wasm_bindgen::JsStringLike, args: &DispatchNamespaceArgs) -> Result<Fetcher, crate::Error>;`
- `pub fn get_with_args_and_options(this: &DispatchNamespace, name: &str, args: &DispatchNamespaceArgs, options: &DynamicDispatchOptions) -> Result<Fetcher, crate::Error>;` → `pub fn get_with_args_and_options(this: &DispatchNamespace, name: impl ::wasm_bindgen::JsStringLike, args: &DispatchNamespaceArgs, options: &DynamicDispatchOptions) -> Result<Fetcher, crate::Error>;`

### `DurableObjectClass`

- `pub type DurableObjectClass<_T: ::wasm_bindgen::JsGeneric = Undefined>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type DurableObjectClass<_T = Undefined>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `DurableObjectFacets`

- `pub fn abort(this: &DurableObjectFacets, name: &str, reason: &JsValue) -> Result<(), crate::Error>;` → `pub fn abort(this: &DurableObjectFacets, name: impl ::wasm_bindgen::JsStringLike, reason: &JsValue) -> Result<(), crate::Error>;`
- `pub fn clone(this: &DurableObjectFacets, src: &str, dst: &str) -> Result<(), crate::Error>;` → `pub fn clone(this: &DurableObjectFacets, src: impl ::wasm_bindgen::JsStringLike, dst: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn delete(this: &DurableObjectFacets, name: &str) -> Result<(), crate::Error>;` → `pub fn delete(this: &DurableObjectFacets, name: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn get<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectFacets, name: &str, get_startup_options: &Function) -> Result<Fetcher<T>, crate::Error>;` → `pub fn get<T>(this: &DurableObjectFacets, name: impl ::wasm_bindgen::JsStringLike, get_startup_options: &Function) -> Result<Fetcher<T>, crate::Error>;`

### `DurableObjectId`

- added → `pub fn jurisdiction_js_string(this: &DurableObjectId) -> Option<JsString>;`
- added → `pub fn name_js_string(this: &DurableObjectId) -> Option<JsString>;`
- added → `pub fn to_string_js_string(this: &DurableObjectId) -> JsString;`

### `DurableObjectListOptions`

- added → `pub fn end_js_string(this: &DurableObjectListOptions) -> Option<JsString>;`
- added → `pub fn prefix_js_string(this: &DurableObjectListOptions) -> Option<JsString>;`
- `pub fn set_end(this: &DurableObjectListOptions, val: &str);` → `pub fn set_end(this: &DurableObjectListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_prefix(this: &DurableObjectListOptions, val: &str);` → `pub fn set_prefix(this: &DurableObjectListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_start(this: &DurableObjectListOptions, val: &str);` → `pub fn set_start(this: &DurableObjectListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_start_after(this: &DurableObjectListOptions, val: &str);` → `pub fn set_start_after(this: &DurableObjectListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn start_after_js_string(this: &DurableObjectListOptions) -> Option<JsString>;`
- added → `pub fn start_js_string(this: &DurableObjectListOptions) -> Option<JsString>;`

### `DurableObjectListOptionsBuilder`

- `pub fn end(self, val: &str) -> Self;` → `pub fn end(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn prefix(self, val: &str) -> Self;` → `pub fn prefix(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn start(self, val: &str) -> Self;` → `pub fn start(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn start_after(self, val: &str) -> Self;` → `pub fn start_after(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `DurableObjectNamespace`

- `pub fn get<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectNamespace<T>, id: &DurableObjectId) -> Result<DurableObjectStub<T>, crate::Error>;` → `pub fn get<T>(this: &DurableObjectNamespace<T>, id: &DurableObjectId) -> Result<DurableObjectStub<T>, crate::Error>;`
- `pub fn get_by_name<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectNamespace<T>, name: &str) -> Result<DurableObjectStub<T>, crate::Error>;` → `pub fn get_by_name<T>(this: &DurableObjectNamespace<T>, name: impl ::wasm_bindgen::JsStringLike) -> Result<DurableObjectStub<T>, crate::Error>;`
- `pub fn get_by_name_with_options<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectNamespace<T>, name: &str, options: &DurableObjectNamespaceGetDurableObjectOptions) -> Result<DurableObjectStub<T>, crate::Error>;` → `pub fn get_by_name_with_options<T>(this: &DurableObjectNamespace<T>, name: impl ::wasm_bindgen::JsStringLike, options: &DurableObjectNamespaceGetDurableObjectOptions) -> Result<DurableObjectStub<T>, crate::Error>;`
- `pub fn get_with_options<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectNamespace<T>, id: &DurableObjectId, options: &DurableObjectNamespaceGetDurableObjectOptions) -> Result<DurableObjectStub<T>, crate::Error>;` → `pub fn get_with_options<T>(this: &DurableObjectNamespace<T>, id: &DurableObjectId, options: &DurableObjectNamespaceGetDurableObjectOptions) -> Result<DurableObjectStub<T>, crate::Error>;`
- `pub fn id_from_name<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectNamespace<T>, name: &str) -> Result<DurableObjectId, crate::Error>;` → `pub fn id_from_name<T>(this: &DurableObjectNamespace<T>, name: impl ::wasm_bindgen::JsStringLike) -> Result<DurableObjectId, crate::Error>;`
- `pub fn id_from_string<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectNamespace<T>, id: &str) -> Result<DurableObjectId, crate::Error>;` → `pub fn id_from_string<T>(this: &DurableObjectNamespace<T>, id: impl ::wasm_bindgen::JsStringLike) -> Result<DurableObjectId, crate::Error>;`
- `pub fn jurisdiction<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectNamespace<T>, jurisdiction: DurableObjectJurisdiction) -> Result<DurableObjectNamespace<T>, crate::Error>;` → `pub fn jurisdiction<T>(this: &DurableObjectNamespace<T>, jurisdiction: DurableObjectJurisdiction) -> Result<DurableObjectNamespace<T>, crate::Error>;`
- `pub fn new_unique_id<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectNamespace<T>) -> Result<DurableObjectId, crate::Error>;` → `pub fn new_unique_id<T>(this: &DurableObjectNamespace<T>) -> Result<DurableObjectId, crate::Error>;`
- `pub fn new_unique_id_with_options<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectNamespace<T>, options: &DurableObjectNamespaceNewUniqueIdOptions) -> Result<DurableObjectId, crate::Error>;` → `pub fn new_unique_id_with_options<T>(this: &DurableObjectNamespace<T>, options: &DurableObjectNamespaceNewUniqueIdOptions) -> Result<DurableObjectId, crate::Error>;`
- `pub type DurableObjectNamespace<T: ::wasm_bindgen::JsGeneric = Undefined>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type DurableObjectNamespace<T = Undefined>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `DurableObjectState`

- `pub fn abort<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> Result<(), crate::Error>;` → `pub fn abort<Props>(this: &DurableObjectState<Props>) -> Result<(), crate::Error>;`
- `pub fn abort_with_reason<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, reason: &str) -> Result<(), crate::Error>;` → `pub fn abort_with_reason<Props>(this: &DurableObjectState<Props>, reason: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn accept_web_socket<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, ws: &WebSocket) -> Result<(), crate::Error>;` → `pub fn accept_web_socket<Props>(this: &DurableObjectState<Props>, ws: &WebSocket) -> Result<(), crate::Error>;`
- `pub fn accept_web_socket_with_tags<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, ws: &WebSocket, tags: &[String]) -> Result<(), crate::Error>;` → `pub fn accept_web_socket_with_tags<Props>(this: &DurableObjectState<Props>, ws: &WebSocket, tags: &[impl ::wasm_bindgen::JsStringLike]) -> Result<(), crate::Error>;`
- `pub async fn block_concurrency_while<Props: ::wasm_bindgen::JsGeneric, T: ::wasm_bindgen::JsGeneric + ::wasm_bindgen::convert::FromWasmAbi>(this: &DurableObjectState<Props>, callback: &Function<fn() -> Promise>) -> Result<T, crate::Error>;` → `pub async fn block_concurrency_while<Props, T>(this: &DurableObjectState<Props>, callback: &Function<fn() -> Promise>) -> Result<T, crate::Error>;`
- `pub fn block_concurrency_while_promise<Props: ::wasm_bindgen::JsGeneric, T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, callback: &Function<fn() -> Promise>) -> Result<Promise<T>, crate::Error>;` → `pub fn block_concurrency_while_promise<Props, T>(this: &DurableObjectState<Props>, callback: &Function<fn() -> Promise>) -> Result<Promise<T>, crate::Error>;`
- `pub fn container<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> Option<Container>;` → `pub fn container<Props>(this: &DurableObjectState<Props>) -> Option<Container>;`
- `pub fn exports<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> JsValue;` → `pub fn exports<Props>(this: &DurableObjectState<Props>) -> JsValue;`
- `pub fn facets<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> DurableObjectFacets;` → `pub fn facets<Props>(this: &DurableObjectState<Props>) -> DurableObjectFacets;`
- `pub fn get_hibernatable_web_socket_event_timeout<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> Result<Option<f64>, crate::Error>;` → `pub fn get_hibernatable_web_socket_event_timeout<Props>(this: &DurableObjectState<Props>) -> Result<Option<f64>, crate::Error>;`
- `pub fn get_tags<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, ws: &WebSocket) -> Result<Vec<String>, crate::Error>;` → `pub fn get_tags<Props>(this: &DurableObjectState<Props>, ws: &WebSocket) -> Result<Vec<String>, crate::Error>;`
- added → `pub fn get_tags_js_string<Props>(this: &DurableObjectState<Props>, ws: &WebSocket) -> Result<Array<JsString>, crate::Error>;`
- `pub fn get_web_socket_auto_response<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> Result<Option<WebSocketRequestResponsePair>, crate::Error>;` → `pub fn get_web_socket_auto_response<Props>(this: &DurableObjectState<Props>) -> Result<Option<WebSocketRequestResponsePair>, crate::Error>;`
- `pub fn get_web_socket_auto_response_timestamp<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, ws: &WebSocket) -> Result<Option<Date>, crate::Error>;` → `pub fn get_web_socket_auto_response_timestamp<Props>(this: &DurableObjectState<Props>, ws: &WebSocket) -> Result<Option<Date>, crate::Error>;`
- `pub fn get_web_sockets<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> Result<Vec<WebSocket>, crate::Error>;` → `pub fn get_web_sockets<Props>(this: &DurableObjectState<Props>) -> Result<Vec<WebSocket>, crate::Error>;`
- `pub fn get_web_sockets_with_tag<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, tag: &str) -> Result<Vec<WebSocket>, crate::Error>;` → `pub fn get_web_sockets_with_tag<Props>(this: &DurableObjectState<Props>, tag: impl ::wasm_bindgen::JsStringLike) -> Result<Vec<WebSocket>, crate::Error>;`
- `pub fn id<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> DurableObjectId;` → `pub fn id<Props>(this: &DurableObjectState<Props>) -> DurableObjectId;`
- `pub fn props<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> Props;` → `pub fn props<Props>(this: &DurableObjectState<Props>) -> Props;`
- `pub fn set_container<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, val: &Container);` → `pub fn set_container<Props>(this: &DurableObjectState<Props>, val: &Container);`
- `pub fn set_facets<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, val: &DurableObjectFacets);` → `pub fn set_facets<Props>(this: &DurableObjectState<Props>, val: &DurableObjectFacets);`
- `pub fn set_hibernatable_web_socket_event_timeout<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> Result<(), crate::Error>;` → `pub fn set_hibernatable_web_socket_event_timeout<Props>(this: &DurableObjectState<Props>) -> Result<(), crate::Error>;`
- `pub fn set_hibernatable_web_socket_event_timeout_with_timeout_ms<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, timeout_ms: f64) -> Result<(), crate::Error>;` → `pub fn set_hibernatable_web_socket_event_timeout_with_timeout_ms<Props>(this: &DurableObjectState<Props>, timeout_ms: f64) -> Result<(), crate::Error>;`
- `pub fn set_web_socket_auto_response<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> Result<(), crate::Error>;` → `pub fn set_web_socket_auto_response<Props>(this: &DurableObjectState<Props>) -> Result<(), crate::Error>;`
- `pub fn set_web_socket_auto_response_with_maybe_req_resp<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, maybe_req_resp: &WebSocketRequestResponsePair) -> Result<(), crate::Error>;` → `pub fn set_web_socket_auto_response_with_maybe_req_resp<Props>(this: &DurableObjectState<Props>, maybe_req_resp: &WebSocketRequestResponsePair) -> Result<(), crate::Error>;`
- `pub fn storage<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>) -> DurableObjectStorage;` → `pub fn storage<Props>(this: &DurableObjectState<Props>) -> DurableObjectStorage;`
- `pub fn wait_until<Props: ::wasm_bindgen::JsGeneric>(this: &DurableObjectState<Props>, promise: &Promise) -> Result<(), crate::Error>;` → `pub fn wait_until<Props>(this: &DurableObjectState<Props>, promise: &Promise) -> Result<(), crate::Error>;`
- `pub type DurableObjectState<Props: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type DurableObjectState<Props = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `DurableObjectStorage`

- `pub async fn delete(this: &DurableObjectStorage, key: &str) -> Result<bool, crate::Error>;` → `pub async fn delete(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike) -> Result<bool, crate::Error>;`
- `pub fn delete_promise(this: &DurableObjectStorage, key: &str) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn delete_promise(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Boolean>, crate::Error>;`
- `pub async fn delete_with_key_and_options(this: &DurableObjectStorage, key: &str, options: &DurableObjectPutOptions) -> Result<bool, crate::Error>;` → `pub async fn delete_with_key_and_options(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike, options: &DurableObjectPutOptions) -> Result<bool, crate::Error>;`
- `pub fn delete_with_key_and_options_promise(this: &DurableObjectStorage, key: &str, options: &DurableObjectPutOptions) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn delete_with_key_and_options_promise(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike, options: &DurableObjectPutOptions) -> Result<Promise<Boolean>, crate::Error>;`
- `pub async fn delete_with_keys(this: &DurableObjectStorage, keys: &[String]) -> Result<usize, crate::Error>;` → `pub async fn delete_with_keys(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<usize, crate::Error>;`
- `pub async fn delete_with_keys_and_options(this: &DurableObjectStorage, keys: &[String], options: &DurableObjectPutOptions) -> Result<usize, crate::Error>;` → `pub async fn delete_with_keys_and_options(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectPutOptions) -> Result<usize, crate::Error>;`
- `pub fn delete_with_keys_and_options_promise(this: &DurableObjectStorage, keys: &[String], options: &DurableObjectPutOptions) -> Result<Promise<usize>, crate::Error>;` → `pub fn delete_with_keys_and_options_promise(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectPutOptions) -> Result<Promise<usize>, crate::Error>;`
- `pub fn delete_with_keys_promise(this: &DurableObjectStorage, keys: &[String]) -> Result<Promise<usize>, crate::Error>;` → `pub fn delete_with_keys_promise(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<usize>, crate::Error>;`
- `pub async fn get<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, key: &str) -> Result<JsOption<T>, crate::Error>;` → `pub async fn get<T>(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike) -> Result<JsOption<T>, crate::Error>;`
- added → `pub async fn get_bookmark_for_time_js_string(this: &DurableObjectStorage, timestamp: f64) -> Result<JsString, crate::Error>;`
- added → `pub async fn get_bookmark_for_time_with_date_js_string(this: &DurableObjectStorage, timestamp: &Date) -> Result<JsString, crate::Error>;`
- added → `pub async fn get_current_bookmark_js_string(this: &DurableObjectStorage) -> Result<JsString, crate::Error>;`
- `pub fn get_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, key: &str) -> Result<Promise<JsOption<T>>, crate::Error>;` → `pub fn get_promise<T>(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsOption<T>>, crate::Error>;`
- `pub async fn get_with_key_and_options<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, key: &str, options: &DurableObjectGetOptions) -> Result<JsOption<T>, crate::Error>;` → `pub async fn get_with_key_and_options<T>(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike, options: &DurableObjectGetOptions) -> Result<JsOption<T>, crate::Error>;`
- `pub fn get_with_key_and_options_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, key: &str, options: &DurableObjectGetOptions) -> Result<Promise<JsOption<T>>, crate::Error>;` → `pub fn get_with_key_and_options_promise<T>(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike, options: &DurableObjectGetOptions) -> Result<Promise<JsOption<T>>, crate::Error>;`
- `pub async fn get_with_keys(this: &DurableObjectStorage, keys: &[String]) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_keys(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub async fn get_with_keys_and_options(this: &DurableObjectStorage, keys: &[String], options: &DurableObjectGetOptions) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_keys_and_options(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectGetOptions) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub fn get_with_keys_and_options_promise(this: &DurableObjectStorage, keys: &[String], options: &DurableObjectGetOptions) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_keys_and_options_promise(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectGetOptions) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `pub fn get_with_keys_promise(this: &DurableObjectStorage, keys: &[String]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_keys_promise(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `pub async fn on_next_session_restore_bookmark(this: &DurableObjectStorage, bookmark: &str) -> Result<String, crate::Error>;` → `pub async fn on_next_session_restore_bookmark(this: &DurableObjectStorage, bookmark: impl ::wasm_bindgen::JsStringLike) -> Result<String, crate::Error>;`
- added → `pub async fn on_next_session_restore_bookmark_js_string(this: &DurableObjectStorage, bookmark: impl ::wasm_bindgen::JsStringLike) -> Result<JsString, crate::Error>;`
- `pub fn on_next_session_restore_bookmark_promise(this: &DurableObjectStorage, bookmark: &str) -> Result<Promise<JsString>, crate::Error>;` → `pub fn on_next_session_restore_bookmark_promise(this: &DurableObjectStorage, bookmark: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsString>, crate::Error>;`
- `pub async fn put<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, key: &str, value: &T) -> Result<(), crate::Error>;` → `pub async fn put<T>(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike, value: T) -> Result<(), crate::Error>;`
- `pub fn put_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, key: &str, value: &T) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_promise<T>(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike, value: T) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn put_with_entries<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, entries: &Object<T>) -> Result<(), crate::Error>;` → `pub async fn put_with_entries<T>(this: &DurableObjectStorage, entries: &Object<T>) -> Result<(), crate::Error>;`
- `pub async fn put_with_entries_and_options<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, entries: &Object<T>, options: &DurableObjectPutOptions) -> Result<(), crate::Error>;` → `pub async fn put_with_entries_and_options<T>(this: &DurableObjectStorage, entries: &Object<T>, options: &DurableObjectPutOptions) -> Result<(), crate::Error>;`
- `pub fn put_with_entries_and_options_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, entries: &Object<T>, options: &DurableObjectPutOptions) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_entries_and_options_promise<T>(this: &DurableObjectStorage, entries: &Object<T>, options: &DurableObjectPutOptions) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn put_with_entries_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, entries: &Object<T>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_entries_promise<T>(this: &DurableObjectStorage, entries: &Object<T>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn put_with_key_and_value_and_options<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, key: &str, value: &T, options: &DurableObjectPutOptions) -> Result<(), crate::Error>;` → `pub async fn put_with_key_and_value_and_options<T>(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike, value: T, options: &DurableObjectPutOptions) -> Result<(), crate::Error>;`
- `pub fn put_with_key_and_value_and_options_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, key: &str, value: &T, options: &DurableObjectPutOptions) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_key_and_value_and_options_promise<T>(this: &DurableObjectStorage, key: impl ::wasm_bindgen::JsStringLike, value: T, options: &DurableObjectPutOptions) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn transaction<T: ::wasm_bindgen::JsGeneric + ::wasm_bindgen::convert::FromWasmAbi>(this: &DurableObjectStorage, closure: &Function<fn(DurableObjectTransaction) -> Promise>) -> Result<T, crate::Error>;` → `pub async fn transaction<T>(this: &DurableObjectStorage, closure: &Function<fn(DurableObjectTransaction) -> Promise>) -> Result<T, crate::Error>;`
- `pub fn transaction_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, closure: &Function<fn(DurableObjectTransaction) -> Promise>) -> Result<Promise<T>, crate::Error>;` → `pub fn transaction_promise<T>(this: &DurableObjectStorage, closure: &Function<fn(DurableObjectTransaction) -> Promise>) -> Result<Promise<T>, crate::Error>;`
- `pub fn transaction_sync<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStorage, closure: &Function<fn() -> T>) -> Result<T, crate::Error>;` → `pub fn transaction_sync<T>(this: &DurableObjectStorage, closure: &Function<fn() -> T>) -> Result<T, crate::Error>;`

### `DurableObjectStub`

- `pub fn id<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStub<T>) -> DurableObjectId;` → `pub fn id<T>(this: &DurableObjectStub<T>) -> DurableObjectId;`
- `pub fn name<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectStub<T>) -> Option<String>;` → `pub fn name<T>(this: &DurableObjectStub<T>) -> Option<String>;`
- added → `pub fn name_js_string<T>(this: &DurableObjectStub<T>) -> Option<JsString>;`
- `pub type DurableObjectStub<T: ::wasm_bindgen::JsGeneric = Undefined>; extends(Fetcher, Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type DurableObjectStub<T = Undefined>; extends(Fetcher, Object) derive(Debug, Clone, PartialEq, Eq)`

### `DurableObjectTransaction`

- `pub async fn delete(this: &DurableObjectTransaction, key: &str) -> Result<bool, crate::Error>;` → `pub async fn delete(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike) -> Result<bool, crate::Error>;`
- `pub fn delete_promise(this: &DurableObjectTransaction, key: &str) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn delete_promise(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Boolean>, crate::Error>;`
- `pub async fn delete_with_key_and_options(this: &DurableObjectTransaction, key: &str, options: &DurableObjectPutOptions) -> Result<bool, crate::Error>;` → `pub async fn delete_with_key_and_options(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike, options: &DurableObjectPutOptions) -> Result<bool, crate::Error>;`
- `pub fn delete_with_key_and_options_promise(this: &DurableObjectTransaction, key: &str, options: &DurableObjectPutOptions) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn delete_with_key_and_options_promise(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike, options: &DurableObjectPutOptions) -> Result<Promise<Boolean>, crate::Error>;`
- `pub async fn delete_with_keys(this: &DurableObjectTransaction, keys: &[String]) -> Result<usize, crate::Error>;` → `pub async fn delete_with_keys(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<usize, crate::Error>;`
- `pub async fn delete_with_keys_and_options(this: &DurableObjectTransaction, keys: &[String], options: &DurableObjectPutOptions) -> Result<usize, crate::Error>;` → `pub async fn delete_with_keys_and_options(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectPutOptions) -> Result<usize, crate::Error>;`
- `pub fn delete_with_keys_and_options_promise(this: &DurableObjectTransaction, keys: &[String], options: &DurableObjectPutOptions) -> Result<Promise<usize>, crate::Error>;` → `pub fn delete_with_keys_and_options_promise(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectPutOptions) -> Result<Promise<usize>, crate::Error>;`
- `pub fn delete_with_keys_promise(this: &DurableObjectTransaction, keys: &[String]) -> Result<Promise<usize>, crate::Error>;` → `pub fn delete_with_keys_promise(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<usize>, crate::Error>;`
- `pub async fn get<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, key: &str) -> Result<JsOption<T>, crate::Error>;` → `pub async fn get<T>(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike) -> Result<JsOption<T>, crate::Error>;`
- `pub fn get_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, key: &str) -> Result<Promise<JsOption<T>>, crate::Error>;` → `pub fn get_promise<T>(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsOption<T>>, crate::Error>;`
- `pub async fn get_with_key_and_options<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, key: &str, options: &DurableObjectGetOptions) -> Result<JsOption<T>, crate::Error>;` → `pub async fn get_with_key_and_options<T>(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike, options: &DurableObjectGetOptions) -> Result<JsOption<T>, crate::Error>;`
- `pub fn get_with_key_and_options_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, key: &str, options: &DurableObjectGetOptions) -> Result<Promise<JsOption<T>>, crate::Error>;` → `pub fn get_with_key_and_options_promise<T>(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike, options: &DurableObjectGetOptions) -> Result<Promise<JsOption<T>>, crate::Error>;`
- `pub async fn get_with_keys(this: &DurableObjectTransaction, keys: &[String]) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_keys(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub async fn get_with_keys_and_options(this: &DurableObjectTransaction, keys: &[String], options: &DurableObjectGetOptions) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_keys_and_options(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectGetOptions) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub fn get_with_keys_and_options_promise(this: &DurableObjectTransaction, keys: &[String], options: &DurableObjectGetOptions) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_keys_and_options_promise(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectGetOptions) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `pub fn get_with_keys_promise(this: &DurableObjectTransaction, keys: &[String]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_keys_promise(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `pub async fn put<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, key: &str, value: &T) -> Result<(), crate::Error>;` → `pub async fn put<T>(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike, value: T) -> Result<(), crate::Error>;`
- `pub fn put_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, key: &str, value: &T) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_promise<T>(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike, value: T) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn put_with_entries<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, entries: &Object<T>) -> Result<(), crate::Error>;` → `pub async fn put_with_entries<T>(this: &DurableObjectTransaction, entries: &Object<T>) -> Result<(), crate::Error>;`
- `pub async fn put_with_entries_and_options<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, entries: &Object<T>, options: &DurableObjectPutOptions) -> Result<(), crate::Error>;` → `pub async fn put_with_entries_and_options<T>(this: &DurableObjectTransaction, entries: &Object<T>, options: &DurableObjectPutOptions) -> Result<(), crate::Error>;`
- `pub fn put_with_entries_and_options_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, entries: &Object<T>, options: &DurableObjectPutOptions) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_entries_and_options_promise<T>(this: &DurableObjectTransaction, entries: &Object<T>, options: &DurableObjectPutOptions) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn put_with_entries_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, entries: &Object<T>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_entries_promise<T>(this: &DurableObjectTransaction, entries: &Object<T>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn put_with_key_and_value_and_options<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, key: &str, value: &T, options: &DurableObjectPutOptions) -> Result<(), crate::Error>;` → `pub async fn put_with_key_and_value_and_options<T>(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike, value: T, options: &DurableObjectPutOptions) -> Result<(), crate::Error>;`
- `pub fn put_with_key_and_value_and_options_promise<T: ::wasm_bindgen::JsGeneric>(this: &DurableObjectTransaction, key: &str, value: &T, options: &DurableObjectPutOptions) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_key_and_value_and_options_promise<T>(this: &DurableObjectTransaction, key: impl ::wasm_bindgen::JsStringLike, value: T, options: &DurableObjectPutOptions) -> Result<Promise<Undefined>, crate::Error>;`

### `EmailAddress`

- added → `pub fn email_js_string(this: &EmailAddress) -> JsString;`
- added → `pub fn name_js_string(this: &EmailAddress) -> JsString;`
- `pub fn new(name: &str, email: &str) -> EmailAddress;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike, email: impl ::wasm_bindgen::JsStringLike) -> EmailAddress;`
- `pub fn set_email(this: &EmailAddress, val: &str);` → `pub fn set_email(this: &EmailAddress, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &EmailAddress, val: &str);` → `pub fn set_name(this: &EmailAddress, val: impl ::wasm_bindgen::JsStringLike);`

### `EmailAttachment`

- `pub fn builder_attachment(filename: &str, r#type: &str, content: &str) -> EmailAttachmentBuilder;` → `pub fn builder_attachment(filename: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike, content: impl ::wasm_bindgen::JsStringLike) -> EmailAttachmentBuilder;`
- `pub fn builder_attachment_with_array_buffer(filename: &str, r#type: &str, content: &ArrayBuffer) -> EmailAttachmentBuilder;` → `pub fn builder_attachment_with_array_buffer(filename: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike, content: &ArrayBuffer) -> EmailAttachmentBuilder;`
- `pub fn builder_attachment_with_typed_array<T: ::js_sys::TypedArray>(filename: &str, r#type: &str, content: &T) -> EmailAttachmentBuilder;` → `pub fn builder_attachment_with_typed_array<T: ::js_sys::TypedArray>(filename: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike, content: &T) -> EmailAttachmentBuilder where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- added → `pub fn content_id_js_string(this: &EmailAttachment) -> Option<JsString>;`
- added → `pub fn filename_js_string(this: &EmailAttachment) -> JsString;`
- `pub fn new_attachment(filename: &str, r#type: &str, content: &str) -> EmailAttachment;` → `pub fn new_attachment(filename: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike, content: impl ::wasm_bindgen::JsStringLike) -> EmailAttachment;`
- `pub fn new_attachment_with_array_buffer(filename: &str, r#type: &str, content: &ArrayBuffer) -> EmailAttachment;` → `pub fn new_attachment_with_array_buffer(filename: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike, content: &ArrayBuffer) -> EmailAttachment;`
- `pub fn new_attachment_with_typed_array<T: ::js_sys::TypedArray>(filename: &str, r#type: &str, content: &T) -> EmailAttachment;` → `pub fn new_attachment_with_typed_array<T: ::js_sys::TypedArray>(filename: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike, content: &T) -> EmailAttachment where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn new_inline(content_id: Option<&str>, filename: &str, r#type: &str, content: &str) -> EmailAttachment;` → `pub fn new_inline(content_id: Option<impl ::wasm_bindgen::JsStringLike>, filename: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike, content: impl ::wasm_bindgen::JsStringLike) -> EmailAttachment;`
- `pub fn new_inline_with_array_buffer(content_id: Option<&str>, filename: &str, r#type: &str, content: &ArrayBuffer) -> EmailAttachment;` → `pub fn new_inline_with_array_buffer(content_id: Option<impl ::wasm_bindgen::JsStringLike>, filename: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike, content: &ArrayBuffer) -> EmailAttachment;`
- `pub fn new_inline_with_typed_array<T: ::js_sys::TypedArray>(content_id: Option<&str>, filename: &str, r#type: &str, content: &T) -> EmailAttachment;` → `pub fn new_inline_with_typed_array<T: ::js_sys::TypedArray>(content_id: Option<impl ::wasm_bindgen::JsStringLike>, filename: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike, content: &T) -> EmailAttachment where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn set_content(this: &EmailAttachment, val: &str);` → `pub fn set_content(this: &EmailAttachment, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_content_id(this: &EmailAttachment, val: Option<&str>);` → `pub fn set_content_id(this: &EmailAttachment, val: Option<impl ::wasm_bindgen::JsStringLike>);`
- `pub fn set_filename(this: &EmailAttachment, val: &str);` → `pub fn set_filename(this: &EmailAttachment, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_type(this: &EmailAttachment, val: &str);` → `pub fn set_type(this: &EmailAttachment, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn type__js_string(this: &EmailAttachment) -> JsString;`

### `EmailAttachmentBuilder`

- `pub fn content_id(self, val: Option<&str>) -> Self;` → `pub fn content_id(self, val: Option<impl ::wasm_bindgen::JsStringLike>) -> Self;`

### `EmailDestinations`

- `pub fn set_bcc(this: &EmailDestinations, val: &str);` → `pub fn set_bcc(this: &EmailDestinations, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cc(this: &EmailDestinations, val: &str);` → `pub fn set_cc(this: &EmailDestinations, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_to(this: &EmailDestinations, val: &str);` → `pub fn set_to(this: &EmailDestinations, val: impl ::wasm_bindgen::JsStringLike);`

### `EmailDestinationsBuilder`

- `pub fn bcc(self, val: &str) -> Self;` → `pub fn bcc(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn cc(self, val: &str) -> Self;` → `pub fn cc(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn to(self, val: &str) -> Self;` → `pub fn to(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `EmailMessageBuilder`

- `pub fn builder(from: &str, subject: &str) -> EmailMessageBuilderBuilder;` → `pub fn builder(from: impl ::wasm_bindgen::JsStringLike, subject: impl ::wasm_bindgen::JsStringLike) -> EmailMessageBuilderBuilder;`
- `pub fn builder_with_email_address(from: &EmailAddress, subject: &str) -> EmailMessageBuilderBuilder;` → `pub fn builder_with_email_address(from: &EmailAddress, subject: impl ::wasm_bindgen::JsStringLike) -> EmailMessageBuilderBuilder;`
- `pub fn new(from: &str, subject: &str) -> EmailMessageBuilder;` → `pub fn new(from: impl ::wasm_bindgen::JsStringLike, subject: impl ::wasm_bindgen::JsStringLike) -> EmailMessageBuilder;`
- `pub fn new_with_email_address(from: &EmailAddress, subject: &str) -> EmailMessageBuilder;` → `pub fn new_with_email_address(from: &EmailAddress, subject: impl ::wasm_bindgen::JsStringLike) -> EmailMessageBuilder;`
- `pub fn set_bcc(this: &EmailMessageBuilder, val: &str);` → `pub fn set_bcc(this: &EmailMessageBuilder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cc(this: &EmailMessageBuilder, val: &str);` → `pub fn set_cc(this: &EmailMessageBuilder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_to(this: &EmailMessageBuilder, val: &str);` → `pub fn set_to(this: &EmailMessageBuilder, val: impl ::wasm_bindgen::JsStringLike);`

### `EmailMessageBuilderBuilder`

- `pub fn bcc(self, val: &str) -> Self;` → `pub fn bcc(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn cc(self, val: &str) -> Self;` → `pub fn cc(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn html(self, val: &str) -> Self;` → `pub fn html(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn reply_to(self, val: &str) -> Self;` → `pub fn reply_to(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn text(self, val: &str) -> Self;` → `pub fn text(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn to(self, val: &str) -> Self;` → `pub fn to(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `EmailReplyMessageBuilder`

- `pub fn builder(from: &str, subject: &str) -> EmailReplyMessageBuilderBuilder;` → `pub fn builder(from: impl ::wasm_bindgen::JsStringLike, subject: impl ::wasm_bindgen::JsStringLike) -> EmailReplyMessageBuilderBuilder;`
- `pub fn builder_with_email_address(from: &EmailAddress, subject: &str) -> EmailReplyMessageBuilderBuilder;` → `pub fn builder_with_email_address(from: &EmailAddress, subject: impl ::wasm_bindgen::JsStringLike) -> EmailReplyMessageBuilderBuilder;`
- added → `pub fn html_js_string(this: &EmailReplyMessageBuilder) -> Option<JsString>;`
- `pub fn new(from: &str, subject: &str) -> EmailReplyMessageBuilder;` → `pub fn new(from: impl ::wasm_bindgen::JsStringLike, subject: impl ::wasm_bindgen::JsStringLike) -> EmailReplyMessageBuilder;`
- `pub fn new_with_email_address(from: &EmailAddress, subject: &str) -> EmailReplyMessageBuilder;` → `pub fn new_with_email_address(from: &EmailAddress, subject: impl ::wasm_bindgen::JsStringLike) -> EmailReplyMessageBuilder;`
- `pub fn set_from(this: &EmailReplyMessageBuilder, val: &str);` → `pub fn set_from(this: &EmailReplyMessageBuilder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_html(this: &EmailReplyMessageBuilder, val: &str);` → `pub fn set_html(this: &EmailReplyMessageBuilder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_reply_to(this: &EmailReplyMessageBuilder, val: &str);` → `pub fn set_reply_to(this: &EmailReplyMessageBuilder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_subject(this: &EmailReplyMessageBuilder, val: &str);` → `pub fn set_subject(this: &EmailReplyMessageBuilder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_text(this: &EmailReplyMessageBuilder, val: &str);` → `pub fn set_text(this: &EmailReplyMessageBuilder, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn subject_js_string(this: &EmailReplyMessageBuilder) -> JsString;`
- added → `pub fn text_js_string(this: &EmailReplyMessageBuilder) -> Option<JsString>;`

### `EmailReplyMessageBuilderBuilder`

- `pub fn html(self, val: &str) -> Self;` → `pub fn html(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn reply_to(self, val: &str) -> Self;` → `pub fn reply_to(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn text(self, val: &str) -> Self;` → `pub fn text(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `EmailSendResult`

- added → `pub fn message_id_js_string(this: &EmailSendResult) -> JsString;`
- `pub fn new(message_id: &str) -> EmailSendResult;` → `pub fn new(message_id: impl ::wasm_bindgen::JsStringLike) -> EmailSendResult;`
- `pub fn set_message_id(this: &EmailSendResult, val: &str);` → `pub fn set_message_id(this: &EmailSendResult, val: impl ::wasm_bindgen::JsStringLike);`

### `ErrorEvent`

- added → `pub fn filename_js_string(this: &ErrorEvent) -> JsString;`
- added → `pub fn message_js_string(this: &ErrorEvent) -> JsString;`
- `pub fn new(r#type: &str) -> Result<ErrorEvent, crate::Error>;` → `pub fn new(r#type: impl ::wasm_bindgen::JsStringLike) -> Result<ErrorEvent, crate::Error>;`
- `pub fn new_with_init(r#type: &str, init: &ErrorEventErrorEventInit) -> Result<ErrorEvent, crate::Error>;` → `pub fn new_with_init(r#type: impl ::wasm_bindgen::JsStringLike, init: &ErrorEventErrorEventInit) -> Result<ErrorEvent, crate::Error>;`

### `ErrorEventErrorEventInit`

- added → `pub fn filename_js_string(this: &ErrorEventErrorEventInit) -> Option<JsString>;`
- added → `pub fn message_js_string(this: &ErrorEventErrorEventInit) -> Option<JsString>;`
- `pub fn set_filename(this: &ErrorEventErrorEventInit, val: &str);` → `pub fn set_filename(this: &ErrorEventErrorEventInit, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_message(this: &ErrorEventErrorEventInit, val: &str);` → `pub fn set_message(this: &ErrorEventErrorEventInit, val: impl ::wasm_bindgen::JsStringLike);`

### `ErrorEventErrorEventInitBuilder`

- `pub fn filename(self, val: &str) -> Self;` → `pub fn filename(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn message(self, val: &str) -> Self;` → `pub fn message(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `Event`

- `pub fn new(r#type: &str) -> Result<Event, crate::Error>;` → `pub fn new(r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Event, crate::Error>;`
- `pub fn new_with_init(r#type: &str, init: &EventInit) -> Result<Event, crate::Error>;` → `pub fn new_with_init(r#type: impl ::wasm_bindgen::JsStringLike, init: &EventInit) -> Result<Event, crate::Error>;`
- added → `pub fn type__js_string(this: &Event) -> JsString;`

### `EventCounts`

- `pub fn get(this: &EventCounts, event_type: &str) -> Result<Option<f64>, crate::Error>;` → `pub fn get(this: &EventCounts, event_type: impl ::wasm_bindgen::JsStringLike) -> Result<Option<f64>, crate::Error>;`
- `pub fn has(this: &EventCounts, event_type: &str) -> Result<bool, crate::Error>;` → `pub fn has(this: &EventCounts, event_type: impl ::wasm_bindgen::JsStringLike) -> Result<bool, crate::Error>;`

### `EventListenerObject`

- `pub fn handle_event<EventType: ::wasm_bindgen::JsGeneric>(this: &EventListenerObject<EventType>, event: &EventType) -> Result<(), crate::Error>;` → `pub fn handle_event<EventType>(this: &EventListenerObject<EventType>, event: EventType) -> Result<(), crate::Error>;`
- `pub type EventListenerObject<EventType: ::wasm_bindgen::JsGeneric = Event>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type EventListenerObject<EventType = Event>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `EventTarget`

- `pub fn add_event_listener<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &Function<fn(JsValue) -> Undefined>) -> Result<(), crate::Error>;` → `pub fn add_event_listener<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &Function<fn(JsValue) -> Undefined>) -> Result<(), crate::Error>;`
- `pub fn add_event_listener_with_event_listener_object<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &EventListenerObject<JsValue>) -> Result<(), crate::Error>;` → `pub fn add_event_listener_with_event_listener_object<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &EventListenerObject<JsValue>) -> Result<(), crate::Error>;`
- `pub fn add_event_listener_with_event_listener_object_and_bool<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &EventListenerObject<JsValue>, options: bool) -> Result<(), crate::Error>;` → `pub fn add_event_listener_with_event_listener_object_and_bool<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &EventListenerObject<JsValue>, options: bool) -> Result<(), crate::Error>;`
- `pub fn add_event_listener_with_event_listener_object_and_event_target_add_event_listener_options<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &EventListenerObject<JsValue>, options: &EventTargetAddEventListenerOptions) -> Result<(), crate::Error>;` → `pub fn add_event_listener_with_event_listener_object_and_event_target_add_event_listener_options<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &EventListenerObject<JsValue>, options: &EventTargetAddEventListenerOptions) -> Result<(), crate::Error>;`
- `pub fn add_event_listener_with_function_and_bool<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &Function<fn(JsValue) -> Undefined>, options: bool) -> Result<(), crate::Error>;` → `pub fn add_event_listener_with_function_and_bool<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &Function<fn(JsValue) -> Undefined>, options: bool) -> Result<(), crate::Error>;`
- `pub fn add_event_listener_with_function_and_event_target_add_event_listener_options<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &Function<fn(JsValue) -> Undefined>, options: &EventTargetAddEventListenerOptions) -> Result<(), crate::Error>;` → `pub fn add_event_listener_with_function_and_event_target_add_event_listener_options<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &Function<fn(JsValue) -> Undefined>, options: &EventTargetAddEventListenerOptions) -> Result<(), crate::Error>;`
- `pub fn dispatch_event<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, event: &JsValue) -> Result<bool, crate::Error>;` → `pub fn dispatch_event<EventMap>(this: &EventTarget<EventMap>, event: &JsValue) -> Result<bool, crate::Error>;`
- `pub fn new() -> Result<EventTarget, crate::Error>;` → `pub fn new<EventMap>() -> Result<EventTarget<EventMap>, crate::Error>;`
- `pub fn remove_event_listener<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &Function<fn(JsValue) -> Undefined>) -> Result<(), crate::Error>;` → `pub fn remove_event_listener<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &Function<fn(JsValue) -> Undefined>) -> Result<(), crate::Error>;`
- `pub fn remove_event_listener_with_event_listener_object<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &EventListenerObject<JsValue>) -> Result<(), crate::Error>;` → `pub fn remove_event_listener_with_event_listener_object<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &EventListenerObject<JsValue>) -> Result<(), crate::Error>;`
- `pub fn remove_event_listener_with_event_listener_object_and_bool<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &EventListenerObject<JsValue>, options: bool) -> Result<(), crate::Error>;` → `pub fn remove_event_listener_with_event_listener_object_and_bool<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &EventListenerObject<JsValue>, options: bool) -> Result<(), crate::Error>;`
- `pub fn remove_event_listener_with_event_listener_object_and_event_target_event_listener_options<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &EventListenerObject<JsValue>, options: &EventTargetEventListenerOptions) -> Result<(), crate::Error>;` → `pub fn remove_event_listener_with_event_listener_object_and_event_target_event_listener_options<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &EventListenerObject<JsValue>, options: &EventTargetEventListenerOptions) -> Result<(), crate::Error>;`
- `pub fn remove_event_listener_with_function_and_bool<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &Function<fn(JsValue) -> Undefined>, options: bool) -> Result<(), crate::Error>;` → `pub fn remove_event_listener_with_function_and_bool<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &Function<fn(JsValue) -> Undefined>, options: bool) -> Result<(), crate::Error>;`
- `pub fn remove_event_listener_with_function_and_event_target_event_listener_options<EventMap: ::wasm_bindgen::JsGeneric>(this: &EventTarget<EventMap>, r#type: &str, handler: &Function<fn(JsValue) -> Undefined>, options: &EventTargetEventListenerOptions) -> Result<(), crate::Error>;` → `pub fn remove_event_listener_with_function_and_event_target_event_listener_options<EventMap>(this: &EventTarget<EventMap>, r#type: impl ::wasm_bindgen::JsStringLike, handler: &Function<fn(JsValue) -> Undefined>, options: &EventTargetEventListenerOptions) -> Result<(), crate::Error>;`
- `pub type EventTarget<EventMap: ::wasm_bindgen::JsGeneric = Object<Event>>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type EventTarget<EventMap = Object<Event>>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `ExecutionContext`

- `pub fn abort<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>) -> Result<(), crate::Error>;` → `pub fn abort<Props>(this: &ExecutionContext<Props>) -> Result<(), crate::Error>;`
- `pub fn abort_with_reason<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>, reason: &JsValue) -> Result<(), crate::Error>;` → `pub fn abort_with_reason<Props>(this: &ExecutionContext<Props>, reason: &JsValue) -> Result<(), crate::Error>;`
- `pub fn access<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>) -> Option<CloudflareAccessContext>;` → `pub fn access<Props>(this: &ExecutionContext<Props>) -> Option<CloudflareAccessContext>;`
- `pub fn cache<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>) -> Option<CacheContext>;` → `pub fn cache<Props>(this: &ExecutionContext<Props>) -> Option<CacheContext>;`
- `pub fn exports<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>) -> JsValue;` → `pub fn exports<Props>(this: &ExecutionContext<Props>) -> JsValue;`
- `pub fn pass_through_on_exception<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>);` → `pub fn pass_through_on_exception<Props>(this: &ExecutionContext<Props>);`
- `pub fn props<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>) -> Props;` → `pub fn props<Props>(this: &ExecutionContext<Props>) -> Props;`
- `pub fn set_cache<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>, val: &CacheContext);` → `pub fn set_cache<Props>(this: &ExecutionContext<Props>, val: &CacheContext);`
- `pub fn set_tracing<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>, val: &Tracing);` → `pub fn set_tracing<Props>(this: &ExecutionContext<Props>, val: &Tracing);`
- `pub fn tracing<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>) -> Tracing;` → `pub fn tracing<Props>(this: &ExecutionContext<Props>) -> Tracing;`
- `pub fn wait_until<Props: ::wasm_bindgen::JsGeneric>(this: &ExecutionContext<Props>, promise: &Promise);` → `pub fn wait_until<Props>(this: &ExecutionContext<Props>, promise: &Promise);`
- `pub type ExecutionContext<Props: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type ExecutionContext<Props = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `ExportedHandler`

- `pub fn connect<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(Socket, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;` → `pub fn connect<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(Socket, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn email<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(ForwardableEmailMessage, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;` → `pub fn email<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(ForwardableEmailMessage, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn fetch<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(Request<CfHostMetadata, IncomingRequestCfProperties<CfHostMetadata>>, Env, ExecutionContext<Props>) -> JsValue>>;` → `pub fn fetch<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(Request<CfHostMetadata, IncomingRequestCfProperties<CfHostMetadata>>, Env, ExecutionContext<Props>) -> JsValue>>;`
- `pub fn queue<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(MessageBatch<QueueHandlerMessage>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;` → `pub fn queue<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(MessageBatch<QueueHandlerMessage>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn scheduled<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(ScheduledController, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;` → `pub fn scheduled<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(ScheduledController, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn set_connect<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(Socket, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_connect<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(Socket, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_email<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(ForwardableEmailMessage, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_email<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(ForwardableEmailMessage, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_fetch<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(Request<CfHostMetadata, IncomingRequestCfProperties<CfHostMetadata>>, Env, ExecutionContext<Props>) -> JsValue>);` → `pub fn set_fetch<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(Request<CfHostMetadata, IncomingRequestCfProperties<CfHostMetadata>>, Env, ExecutionContext<Props>) -> JsValue>);`
- `pub fn set_queue<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(MessageBatch<QueueHandlerMessage>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_queue<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(MessageBatch<QueueHandlerMessage>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_scheduled<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(ScheduledController, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_scheduled<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(ScheduledController, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_tail<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(Array<TraceItem>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_tail<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(Array<TraceItem>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_tail_stream<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(tail_stream::TailEvent<tail_stream::Onset>, Env, ExecutionContext<Props>) -> JsValue>);` → `pub fn set_tail_stream<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(tail_stream::TailEvent<tail_stream::Onset>, Env, ExecutionContext<Props>) -> JsValue>);`
- `pub fn set_test<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(TestController, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_test<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(TestController, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_trace<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(Array<TraceItem>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_trace<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>, val: &Function<fn(Array<TraceItem>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>);`
- `pub fn tail<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(Array<TraceItem>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;` → `pub fn tail<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(Array<TraceItem>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn tail_stream<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(tail_stream::TailEvent<tail_stream::Onset>, Env, ExecutionContext<Props>) -> JsValue>>;` → `pub fn tail_stream<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(tail_stream::TailEvent<tail_stream::Onset>, Env, ExecutionContext<Props>) -> JsValue>>;`
- `pub fn test<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(TestController, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;` → `pub fn test<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(TestController, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn trace<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(Array<TraceItem>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;` → `pub fn trace<Env, QueueHandlerMessage, CfHostMetadata, Props>(this: &ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>) -> Option<Function<fn(Array<TraceItem>, Env, ExecutionContext<Props>) -> JsOption<Promise<Undefined>>>>;`
- `impl<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric> ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>` → `impl<Env: ::wasm_bindgen::convert::IntoWasmAbi, QueueHandlerMessage: ::wasm_bindgen::convert::IntoWasmAbi, CfHostMetadata: ::wasm_bindgen::convert::IntoWasmAbi, Props: ::wasm_bindgen::convert::IntoWasmAbi> ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>`
- `pub type ExportedHandler<Env: ::wasm_bindgen::JsGeneric = JsValue, QueueHandlerMessage: ::wasm_bindgen::JsGeneric = JsValue, CfHostMetadata: ::wasm_bindgen::JsGeneric = JsValue, Props: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type ExportedHandler<Env = JsValue, QueueHandlerMessage = JsValue, CfHostMetadata = JsValue, Props = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `ExportedHandlerBuilder`

- `impl<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric> ExportedHandlerBuilder<Env, QueueHandlerMessage, CfHostMetadata, Props>` → `impl<Env: ::wasm_bindgen::convert::IntoWasmAbi, QueueHandlerMessage: ::wasm_bindgen::convert::IntoWasmAbi, CfHostMetadata: ::wasm_bindgen::convert::IntoWasmAbi, Props: ::wasm_bindgen::convert::IntoWasmAbi> ExportedHandlerBuilder<Env, QueueHandlerMessage, CfHostMetadata, Props>`
- `impl<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric> From<ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>> for ExportedHandlerBuilder<Env, QueueHandlerMessage, CfHostMetadata, Props>` → `impl<Env: ::wasm_bindgen::convert::IntoWasmAbi, QueueHandlerMessage: ::wasm_bindgen::convert::IntoWasmAbi, CfHostMetadata: ::wasm_bindgen::convert::IntoWasmAbi, Props: ::wasm_bindgen::convert::IntoWasmAbi> From<ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props>> for ExportedHandlerBuilder<Env, QueueHandlerMessage, CfHostMetadata, Props>`
- `pub struct ExportedHandlerBuilder<Env: ::wasm_bindgen::JsGeneric, QueueHandlerMessage: ::wasm_bindgen::JsGeneric, CfHostMetadata: ::wasm_bindgen::JsGeneric, Props: ::wasm_bindgen::JsGeneric> { inner: ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props> }` → `pub struct ExportedHandlerBuilder<Env, QueueHandlerMessage, CfHostMetadata, Props> { inner: ExportedHandler<Env, QueueHandlerMessage, CfHostMetadata, Props> }`

### `FacetStartupOptions`

- `pub fn class<T: ::wasm_bindgen::JsGeneric>(this: &FacetStartupOptions<T>) -> DurableObjectClass<T>;` → `pub fn class<T>(this: &FacetStartupOptions<T>) -> DurableObjectClass<T>;`
- `pub fn id<T: ::wasm_bindgen::JsGeneric>(this: &FacetStartupOptions<T>) -> Option<IdKind>;` → `pub fn id<T>(this: &FacetStartupOptions<T>) -> Option<IdKind>;`
- `pub fn set_class<T: ::wasm_bindgen::JsGeneric>(this: &FacetStartupOptions<T>, val: &DurableObjectClass<T>);` → `pub fn set_class<T>(this: &FacetStartupOptions<T>, val: &DurableObjectClass<T>);`
- `pub fn set_id<T: ::wasm_bindgen::JsGeneric>(this: &FacetStartupOptions<T>, val: &DurableObjectId);` → `pub fn set_id<T>(this: &FacetStartupOptions<T>, val: &DurableObjectId);`
- `pub fn set_id_with_str<T: ::wasm_bindgen::JsGeneric>(this: &FacetStartupOptions<T>, val: &str);` → `pub fn set_id_with_str<T>(this: &FacetStartupOptions<T>, val: impl ::wasm_bindgen::JsStringLike);`
- `impl<T: ::wasm_bindgen::JsGeneric> FacetStartupOptions<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> FacetStartupOptions<T>`
- `pub type FacetStartupOptions<T: ::wasm_bindgen::JsGeneric = Undefined>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type FacetStartupOptions<T = Undefined>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `FacetStartupOptionsBuilder`

- `pub fn id_with_str(self, val: &str) -> Self;` → `pub fn id_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `impl<T: ::wasm_bindgen::JsGeneric> FacetStartupOptionsBuilder<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> FacetStartupOptionsBuilder<T>`
- `impl<T: ::wasm_bindgen::JsGeneric> From<FacetStartupOptions<T>> for FacetStartupOptionsBuilder<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> From<FacetStartupOptions<T>> for FacetStartupOptionsBuilder<T>`
- `pub struct FacetStartupOptionsBuilder<T: ::wasm_bindgen::JsGeneric> { inner: FacetStartupOptions<T> }` → `pub struct FacetStartupOptionsBuilder<T> { inner: FacetStartupOptions<T> }`

### `Fetcher`

- `pub fn connect<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, address: &SocketAddress) -> Result<Socket, crate::Error>;` → `pub fn connect<T>(this: &Fetcher<T>, address: &SocketAddress) -> Result<Socket, crate::Error>;`
- `pub fn connect_with_socket_address_and_options<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, address: &SocketAddress, options: &SocketOptions) -> Result<Socket, crate::Error>;` → `pub fn connect_with_socket_address_and_options<T>(this: &Fetcher<T>, address: &SocketAddress, options: &SocketOptions) -> Result<Socket, crate::Error>;`
- `pub fn connect_with_str<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, address: &str) -> Result<Socket, crate::Error>;` → `pub fn connect_with_str<T>(this: &Fetcher<T>, address: impl ::wasm_bindgen::JsStringLike) -> Result<Socket, crate::Error>;`
- `pub fn connect_with_str_and_options<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, address: &str, options: &SocketOptions) -> Result<Socket, crate::Error>;` → `pub fn connect_with_str_and_options<T>(this: &Fetcher<T>, address: impl ::wasm_bindgen::JsStringLike, options: &SocketOptions) -> Result<Socket, crate::Error>;`
- `pub async fn fetch<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>) -> Result<Response, crate::Error>;` → `pub async fn fetch<T>(this: &Fetcher<T>, input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>) -> Result<Response, crate::Error>;`
- `pub fn fetch_promise<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>) -> Result<Promise<Response>, crate::Error>;` → `pub fn fetch_promise<T>(this: &Fetcher<T>, input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>) -> Result<Promise<Response>, crate::Error>;`
- `pub async fn fetch_with_request_and_init<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>, init: &RequestInit) -> Result<Response, crate::Error>;` → `pub async fn fetch_with_request_and_init<T>(this: &Fetcher<T>, input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>, init: &RequestInit) -> Result<Response, crate::Error>;`
- `pub fn fetch_with_request_and_init_promise<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>, init: &RequestInit) -> Result<Promise<Response>, crate::Error>;` → `pub fn fetch_with_request_and_init_promise<T>(this: &Fetcher<T>, input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>, init: &RequestInit) -> Result<Promise<Response>, crate::Error>;`
- `pub async fn fetch_with_str<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &str) -> Result<Response, crate::Error>;` → `pub async fn fetch_with_str<T>(this: &Fetcher<T>, input: impl ::wasm_bindgen::JsStringLike) -> Result<Response, crate::Error>;`
- `pub async fn fetch_with_str_and_init<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &str, init: &RequestInit) -> Result<Response, crate::Error>;` → `pub async fn fetch_with_str_and_init<T>(this: &Fetcher<T>, input: impl ::wasm_bindgen::JsStringLike, init: &RequestInit) -> Result<Response, crate::Error>;`
- `pub fn fetch_with_str_and_init_promise<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &str, init: &RequestInit) -> Result<Promise<Response>, crate::Error>;` → `pub fn fetch_with_str_and_init_promise<T>(this: &Fetcher<T>, input: impl ::wasm_bindgen::JsStringLike, init: &RequestInit) -> Result<Promise<Response>, crate::Error>;`
- `pub fn fetch_with_str_promise<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &str) -> Result<Promise<Response>, crate::Error>;` → `pub fn fetch_with_str_promise<T>(this: &Fetcher<T>, input: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Response>, crate::Error>;`
- `pub async fn fetch_with_url<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &URL) -> Result<Response, crate::Error>;` → `pub async fn fetch_with_url<T>(this: &Fetcher<T>, input: &URL) -> Result<Response, crate::Error>;`
- `pub async fn fetch_with_url_and_init<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &URL, init: &RequestInit) -> Result<Response, crate::Error>;` → `pub async fn fetch_with_url_and_init<T>(this: &Fetcher<T>, input: &URL, init: &RequestInit) -> Result<Response, crate::Error>;`
- `pub fn fetch_with_url_and_init_promise<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &URL, init: &RequestInit) -> Result<Promise<Response>, crate::Error>;` → `pub fn fetch_with_url_and_init_promise<T>(this: &Fetcher<T>, input: &URL, init: &RequestInit) -> Result<Promise<Response>, crate::Error>;`
- `pub fn fetch_with_url_promise<T: ::wasm_bindgen::JsGeneric>(this: &Fetcher<T>, input: &URL) -> Result<Promise<Response>, crate::Error>;` → `pub fn fetch_with_url_promise<T>(this: &Fetcher<T>, input: &URL) -> Result<Promise<Response>, crate::Error>;`
- `pub type Fetcher<T: ::wasm_bindgen::JsGeneric = Undefined>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type Fetcher<T = Undefined>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `File`

- added → `pub fn name_js_string(this: &File) -> JsString;`
- `pub fn new(bits: Option<&[JsValue]>, name: &str) -> Result<File, crate::Error>;` → `pub fn new(bits: Option<&[JsValue]>, name: impl ::wasm_bindgen::JsStringLike) -> Result<File, crate::Error>;`
- `pub fn new_with_options(bits: Option<&[JsValue]>, name: &str, options: &FileOptions) -> Result<File, crate::Error>;` → `pub fn new_with_options(bits: Option<&[JsValue]>, name: impl ::wasm_bindgen::JsStringLike, options: &FileOptions) -> Result<File, crate::Error>;`

### `FileOptions`

- `pub fn set_type(this: &FileOptions, val: &str);` → `pub fn set_type(this: &FileOptions, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn type__js_string(this: &FileOptions) -> Option<JsString>;`

### `FileOptionsBuilder`

- `pub fn type_(self, val: &str) -> Self;` → `pub fn type_(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `FixedLengthStream`

- `pub fn new(expected_length: f64) -> Result<FixedLengthStream, crate::Error>;` → `pub fn new(expected_length: impl ::js_sys::JsBigIntOrNumberLike) -> Result<FixedLengthStream, crate::Error>;`
- `pub fn new_with_big_int(expected_length: &BigInt) -> Result<FixedLengthStream, crate::Error>;` → removed
- `pub fn new_with_big_int_and_queuing_strategy(expected_length: &BigInt, queuing_strategy: &IdentityTransformStreamQueuingStrategy) -> Result<FixedLengthStream, crate::Error>;` → removed
- `pub fn new_with_f64_and_queuing_strategy(expected_length: f64, queuing_strategy: &IdentityTransformStreamQueuingStrategy) -> Result<FixedLengthStream, crate::Error>;` → removed
- added → `pub fn new_with_queuing_strategy(expected_length: impl ::js_sys::JsBigIntOrNumberLike, queuing_strategy: &IdentityTransformStreamQueuingStrategy) -> Result<FixedLengthStream, crate::Error>;`

### `FormData`

- `pub fn append(this: &FormData, name: &str, value: &str);` → `pub fn append(this: &FormData, name: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike);`
- `pub fn append_with_blob(this: &FormData, name: &str, value: &Blob);` → `pub fn append_with_blob(this: &FormData, name: impl ::wasm_bindgen::JsStringLike, value: &Blob);`
- `pub fn append_with_blob_and_filename(this: &FormData, name: &str, value: &Blob, filename: &str);` → `pub fn append_with_blob_and_filename(this: &FormData, name: impl ::wasm_bindgen::JsStringLike, value: &Blob, filename: impl ::wasm_bindgen::JsStringLike);`
- `pub fn delete(this: &FormData, name: &str);` → `pub fn delete(this: &FormData, name: impl ::wasm_bindgen::JsStringLike);`
- `pub fn for_each_with_this_arg<This: ::wasm_bindgen::JsGeneric>(this: &FormData, callback: &Function<fn(JsValue, JsString, FormData) -> Undefined>, this_arg: &This) -> Result<(), crate::Error>;` → `pub fn for_each_with_this_arg<This>(this: &FormData, callback: &Function<fn(JsValue, JsString, FormData) -> Undefined>, this_arg: This) -> Result<(), crate::Error>;`
- `pub fn get(this: &FormData, name: &str) -> JsValue;` → `pub fn get(this: &FormData, name: impl ::wasm_bindgen::JsStringLike) -> JsValue;`
- `pub fn get_all(this: &FormData, name: &str) -> Vec<JsValue>;` → `pub fn get_all(this: &FormData, name: impl ::wasm_bindgen::JsStringLike) -> Vec<JsValue>;`
- `pub fn has(this: &FormData, name: &str) -> bool;` → `pub fn has(this: &FormData, name: impl ::wasm_bindgen::JsStringLike) -> bool;`
- `pub fn set(this: &FormData, name: &str, value: &str);` → `pub fn set(this: &FormData, name: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_with_blob(this: &FormData, name: &str, value: &Blob);` → `pub fn set_with_blob(this: &FormData, name: impl ::wasm_bindgen::JsStringLike, value: &Blob);`
- `pub fn set_with_blob_and_filename(this: &FormData, name: &str, value: &Blob, filename: &str);` → `pub fn set_with_blob_and_filename(this: &FormData, name: impl ::wasm_bindgen::JsStringLike, value: &Blob, filename: impl ::wasm_bindgen::JsStringLike);`

### `ForwardableEmailMessage`

- `pub async fn forward(this: &ForwardableEmailMessage, rcpt_to: &str) -> Result<EmailSendResult, crate::Error>;` → `pub async fn forward(this: &ForwardableEmailMessage, rcpt_to: impl ::wasm_bindgen::JsStringLike) -> Result<EmailSendResult, crate::Error>;`
- `pub fn forward_promise(this: &ForwardableEmailMessage, rcpt_to: &str) -> Result<Promise<EmailSendResult>, crate::Error>;` → `pub fn forward_promise(this: &ForwardableEmailMessage, rcpt_to: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<EmailSendResult>, crate::Error>;`
- `pub async fn forward_with_headers(this: &ForwardableEmailMessage, rcpt_to: &str, headers: &Headers) -> Result<EmailSendResult, crate::Error>;` → `pub async fn forward_with_headers(this: &ForwardableEmailMessage, rcpt_to: impl ::wasm_bindgen::JsStringLike, headers: &Headers) -> Result<EmailSendResult, crate::Error>;`
- `pub fn forward_with_headers_promise(this: &ForwardableEmailMessage, rcpt_to: &str, headers: &Headers) -> Result<Promise<EmailSendResult>, crate::Error>;` → `pub fn forward_with_headers_promise(this: &ForwardableEmailMessage, rcpt_to: impl ::wasm_bindgen::JsStringLike, headers: &Headers) -> Result<Promise<EmailSendResult>, crate::Error>;`
- `pub fn set_reject(this: &ForwardableEmailMessage, reason: &str) -> Result<(), crate::Error>;` → `pub fn set_reject(this: &ForwardableEmailMessage, reason: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`

### `Headers`

- `pub fn append(this: &Headers, name: &str, value: &str) -> Result<(), JsException>;` → `pub fn append(this: &Headers, name: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike) -> Result<(), JsException>;`
- `pub fn delete(this: &Headers, name: &str) -> Result<(), JsException>;` → `pub fn delete(this: &Headers, name: impl ::wasm_bindgen::JsStringLike) -> Result<(), JsException>;`
- `pub fn for_each_with_this_arg<This: ::wasm_bindgen::JsGeneric>(this: &Headers, callback: &Function<fn(JsString, JsString, Headers) -> Undefined>, this_arg: &This) -> Result<(), crate::Error>;` → `pub fn for_each_with_this_arg<This>(this: &Headers, callback: &Function<fn(JsString, JsString, Headers) -> Undefined>, this_arg: This) -> Result<(), crate::Error>;`
- `pub fn get(this: &Headers, name: &str) -> Result<Option<String>, JsException>;` → `pub fn get(this: &Headers, name: impl ::wasm_bindgen::JsStringLike) -> Result<Option<String>, JsException>;`
- `pub fn get_all(this: &Headers, name: &str) -> Result<Vec<String>, JsException>;` → `pub fn get_all(this: &Headers, name: impl ::wasm_bindgen::JsStringLike) -> Result<Vec<String>, JsException>;`
- added → `pub fn get_all_js_string(this: &Headers, name: impl ::wasm_bindgen::JsStringLike) -> Result<Array<JsString>, JsException>;`
- added → `pub fn get_js_string(this: &Headers, name: impl ::wasm_bindgen::JsStringLike) -> Result<Option<JsString>, JsException>;`
- added → `pub fn get_set_cookie_js_string(this: &Headers) -> Result<Array<JsString>, crate::Error>;`
- `pub fn has(this: &Headers, name: &str) -> Result<bool, JsException>;` → `pub fn has(this: &Headers, name: impl ::wasm_bindgen::JsStringLike) -> Result<bool, JsException>;`
- `pub fn set(this: &Headers, name: &str, value: &str) -> Result<(), JsException>;` → `pub fn set(this: &Headers, name: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike) -> Result<(), JsException>;`

### `Hyperdrive`

- added → `pub fn connection_string_js_string(this: &Hyperdrive) -> JsString;`
- added → `pub fn database_js_string(this: &Hyperdrive) -> JsString;`
- added → `pub fn host_js_string(this: &Hyperdrive) -> JsString;`
- added → `pub fn ip_js_string(this: &Hyperdrive) -> JsString;`
- added → `pub fn password_js_string(this: &Hyperdrive) -> JsString;`
- added → `pub fn user_js_string(this: &Hyperdrive) -> JsString;`

### `IdentityTransformStreamQueuingStrategy`

- `pub fn set_high_water_mark(this: &IdentityTransformStreamQueuingStrategy, val: f64);` → `pub fn set_high_water_mark(this: &IdentityTransformStreamQueuingStrategy, val: impl ::js_sys::JsBigIntOrNumberLike);`
- `pub fn set_high_water_mark_with_big_int(this: &IdentityTransformStreamQueuingStrategy, val: &BigInt);` → removed

### `IdentityTransformStreamQueuingStrategyBuilder`

- `pub fn high_water_mark(self, val: f64) -> Self;` → `pub fn high_water_mark(self, val: impl ::js_sys::JsBigIntOrNumberLike) -> Self;`
- `pub fn high_water_mark_with_big_int(self, val: &BigInt) -> Self;` → removed

### `IncomingRequestCfProperties`

- `pub fn bot_management<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> IncomingRequestCfPropertiesBotManagementBase;` → `pub fn bot_management<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> IncomingRequestCfPropertiesBotManagementBase;`
- `pub fn builder_0(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_0(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_0_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_0_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_1(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_1(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_1_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_1_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_2(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_2(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_2_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_2_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_3(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_3(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_3_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_3_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_4(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_4(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_4_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_4_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_5(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_5(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn builder_5_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;` → `pub fn builder_5_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfPropertiesBuilder<HostMetadata>;`
- `pub fn city<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;` → `pub fn city<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;`
- added → `pub fn city_js_string<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<JsString>;`
- `pub fn client_trust_score<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> f64;` → `pub fn client_trust_score<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> f64;`
- `pub fn continent<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<ContinentCode>;` → `pub fn continent<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<ContinentCode>;`
- `pub fn country<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<CountryKind>;` → `pub fn country<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<CountryKind>;`
- `pub fn host_metadata<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<HostMetadata>;` → `pub fn host_metadata<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<HostMetadata>;`
- `pub fn is_eu_country<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;` → `pub fn is_eu_country<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;`
- added → `pub fn is_eu_country_js_string<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<JsString>;`
- `pub fn latitude<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;` → `pub fn latitude<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;`
- added → `pub fn latitude_js_string<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<JsString>;`
- `pub fn longitude<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;` → `pub fn longitude<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;`
- added → `pub fn longitude_js_string<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<JsString>;`
- `pub fn metro_code<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;` → `pub fn metro_code<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;`
- added → `pub fn metro_code_js_string<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<JsString>;`
- `pub fn new_0(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_0(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_0_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_0_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_1(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_1(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_1_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_1_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_2(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_2(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_2_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_2_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_3(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_3(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_3_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_3_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_4(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_4(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_4_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_4_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_5(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_5(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuth) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn new_5_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;` → `pub fn new_5_with_incoming_request_cf_properties_tls_client_auth_placeholder(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike, client_trust_score: f64, bot_management: &IncomingRequestCfPropertiesBotManagementBase, tls_client_auth: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> IncomingRequestCfProperties<HostMetadata>;`
- `pub fn postal_code<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;` → `pub fn postal_code<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;`
- added → `pub fn postal_code_js_string<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<JsString>;`
- `pub fn region<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;` → `pub fn region<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;`
- `pub fn region_code<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;` → `pub fn region_code<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;`
- added → `pub fn region_code_js_string<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<JsString>;`
- added → `pub fn region_js_string<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<JsString>;`
- `pub fn set_bot_management<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &IncomingRequestCfPropertiesBotManagementBase);` → `pub fn set_bot_management<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: &IncomingRequestCfPropertiesBotManagementBase);`
- `pub fn set_city<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &str);` → `pub fn set_city<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_client_trust_score<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: f64);` → `pub fn set_client_trust_score<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: f64);`
- `pub fn set_continent<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: ContinentCode);` → `pub fn set_continent<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: ContinentCode);`
- `pub fn set_country<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: Iso3166Alpha2Code);` → `pub fn set_country<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: Iso3166Alpha2Code);`
- `pub fn set_country_with_country_kind<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: CountryKind);` → `pub fn set_country_with_country_kind<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: CountryKind);`
- `pub fn set_host_metadata<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &HostMetadata);` → `pub fn set_host_metadata<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: HostMetadata);`
- `pub fn set_is_eu_country<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &str);` → `pub fn set_is_eu_country<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_latitude<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &str);` → `pub fn set_latitude<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_longitude<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &str);` → `pub fn set_longitude<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_metro_code<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &str);` → `pub fn set_metro_code<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_postal_code<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &str);` → `pub fn set_postal_code<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_region<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &str);` → `pub fn set_region<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_region_code<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &str);` → `pub fn set_region_code<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_timezone<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &str);` → `pub fn set_timezone<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_tls_client_auth<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &IncomingRequestCfPropertiesTLSClientAuth);` → `pub fn set_tls_client_auth<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: &IncomingRequestCfPropertiesTLSClientAuth);`
- `pub fn set_tls_client_auth_with_incoming_request_cf_properties_tls_client_auth_placeholder<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>, val: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder);` → `pub fn set_tls_client_auth_with_incoming_request_cf_properties_tls_client_auth_placeholder<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>, val: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder);`
- `pub fn timezone<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;` → `pub fn timezone<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<String>;`
- added → `pub fn timezone_js_string<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> Option<JsString>;`
- `pub fn tls_client_auth<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfProperties<HostMetadata>) -> TlsClientAuthKind;` → `pub fn tls_client_auth<HostMetadata>(this: &IncomingRequestCfProperties<HostMetadata>) -> TlsClientAuthKind;`
- `impl<HostMetadata: ::wasm_bindgen::JsGeneric> IncomingRequestCfProperties<HostMetadata>` → `impl<HostMetadata: ::wasm_bindgen::convert::IntoWasmAbi> IncomingRequestCfProperties<HostMetadata>`
- `pub type IncomingRequestCfProperties<HostMetadata: ::wasm_bindgen::JsGeneric = JsValue>; extends(IncomingRequestCfPropertiesBase, IncomingRequestCfPropertiesBotManagementEnterprise, IncomingRequestCfPropertiesCloudflareForSaaSEnterprise, IncomingRequestCfPropertiesGeographicInformation, IncomingRequestCfPropertiesCloudflareAccessOrApiShield, Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type IncomingRequestCfProperties<HostMetadata = JsValue>; extends(IncomingRequestCfPropertiesBase, IncomingRequestCfPropertiesBotManagementEnterprise, IncomingRequestCfPropertiesCloudflareForSaaSEnterprise, IncomingRequestCfPropertiesGeographicInformation, IncomingRequestCfPropertiesCloudflareAccessOrApiShield, Object) derive(Debug, Clone, PartialEq, Eq)`

### `IncomingRequestCfPropertiesBase`

- added → `pub fn as_organization_js_string(this: &IncomingRequestCfPropertiesBase) -> Option<JsString>;`
- `pub fn builder_0(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBaseBuilder;` → `pub fn builder_0(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBaseBuilder;`
- `pub fn builder_1(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBaseBuilder;` → `pub fn builder_1(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBaseBuilder;`
- `pub fn builder_2(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBaseBuilder;` → `pub fn builder_2(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBaseBuilder;`
- `pub fn builder_3(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBaseBuilder;` → `pub fn builder_3(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBaseBuilder;`
- `pub fn builder_4(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBaseBuilder;` → `pub fn builder_4(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBaseBuilder;`
- `pub fn builder_5(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBaseBuilder;` → `pub fn builder_5(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBaseBuilder;`
- added → `pub fn client_accept_encoding_js_string(this: &IncomingRequestCfPropertiesBase) -> Option<JsString>;`
- added → `pub fn colo_js_string(this: &IncomingRequestCfPropertiesBase) -> JsString;`
- added → `pub fn http_protocol_js_string(this: &IncomingRequestCfPropertiesBase) -> JsString;`
- `pub fn new_0(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBase;` → `pub fn new_0(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBase;`
- `pub fn new_1(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBase;` → `pub fn new_1(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBase;`
- `pub fn new_2(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBase;` → `pub fn new_2(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBase;`
- `pub fn new_3(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBase;` → `pub fn new_3(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBase;`
- `pub fn new_4(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBase;` → `pub fn new_4(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBase;`
- `pub fn new_5(colo: &str, http_protocol: &str, request_priority: &str, tls_version: &str, tls_cipher: &str) -> IncomingRequestCfPropertiesBase;` → `pub fn new_5(colo: impl ::wasm_bindgen::JsStringLike, http_protocol: impl ::wasm_bindgen::JsStringLike, request_priority: impl ::wasm_bindgen::JsStringLike, tls_version: impl ::wasm_bindgen::JsStringLike, tls_cipher: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesBase;`
- added → `pub fn request_priority_js_string(this: &IncomingRequestCfPropertiesBase) -> JsString;`
- `pub fn set_as_organization(this: &IncomingRequestCfPropertiesBase, val: &str);` → `pub fn set_as_organization(this: &IncomingRequestCfPropertiesBase, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_client_accept_encoding(this: &IncomingRequestCfPropertiesBase, val: &str);` → `pub fn set_client_accept_encoding(this: &IncomingRequestCfPropertiesBase, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_colo(this: &IncomingRequestCfPropertiesBase, val: &str);` → `pub fn set_colo(this: &IncomingRequestCfPropertiesBase, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_http_protocol(this: &IncomingRequestCfPropertiesBase, val: &str);` → `pub fn set_http_protocol(this: &IncomingRequestCfPropertiesBase, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_request_priority(this: &IncomingRequestCfPropertiesBase, val: &str);` → `pub fn set_request_priority(this: &IncomingRequestCfPropertiesBase, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_tls_cipher(this: &IncomingRequestCfPropertiesBase, val: &str);` → `pub fn set_tls_cipher(this: &IncomingRequestCfPropertiesBase, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_tls_version(this: &IncomingRequestCfPropertiesBase, val: &str);` → `pub fn set_tls_version(this: &IncomingRequestCfPropertiesBase, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn tls_cipher_js_string(this: &IncomingRequestCfPropertiesBase) -> JsString;`
- added → `pub fn tls_version_js_string(this: &IncomingRequestCfPropertiesBase) -> JsString;`

### `IncomingRequestCfPropertiesBaseBuilder`

- `pub fn as_organization(self, val: &str) -> Self;` → `pub fn as_organization(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn client_accept_encoding(self, val: &str) -> Self;` → `pub fn client_accept_encoding(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `IncomingRequestCfPropertiesBuilder`

- `pub fn as_organization(self, val: &str) -> Self;` → `pub fn as_organization(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn city(self, val: &str) -> Self;` → `pub fn city(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn client_accept_encoding(self, val: &str) -> Self;` → `pub fn client_accept_encoding(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn host_metadata(self, val: &HostMetadata) -> Self;` → `pub fn host_metadata(self, val: HostMetadata) -> Self;`
- `pub fn is_eu_country(self, val: &str) -> Self;` → `pub fn is_eu_country(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn latitude(self, val: &str) -> Self;` → `pub fn latitude(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn longitude(self, val: &str) -> Self;` → `pub fn longitude(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn metro_code(self, val: &str) -> Self;` → `pub fn metro_code(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn postal_code(self, val: &str) -> Self;` → `pub fn postal_code(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn region(self, val: &str) -> Self;` → `pub fn region(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn region_code(self, val: &str) -> Self;` → `pub fn region_code(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn timezone(self, val: &str) -> Self;` → `pub fn timezone(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `impl<HostMetadata: ::wasm_bindgen::JsGeneric> From<IncomingRequestCfProperties<HostMetadata>> for IncomingRequestCfPropertiesBuilder<HostMetadata>` → `impl<HostMetadata: ::wasm_bindgen::convert::IntoWasmAbi> From<IncomingRequestCfProperties<HostMetadata>> for IncomingRequestCfPropertiesBuilder<HostMetadata>`
- `impl<HostMetadata: ::wasm_bindgen::JsGeneric> IncomingRequestCfPropertiesBuilder<HostMetadata>` → `impl<HostMetadata: ::wasm_bindgen::convert::IntoWasmAbi> IncomingRequestCfPropertiesBuilder<HostMetadata>`
- `pub struct IncomingRequestCfPropertiesBuilder<HostMetadata: ::wasm_bindgen::JsGeneric> { inner: IncomingRequestCfProperties<HostMetadata> }` → `pub struct IncomingRequestCfPropertiesBuilder<HostMetadata> { inner: IncomingRequestCfProperties<HostMetadata> }`

### `IncomingRequestCfPropertiesCloudflareForSaaSEnterprise`

- `pub fn host_metadata<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata>) -> Option<HostMetadata>;` → `pub fn host_metadata<HostMetadata>(this: &IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata>) -> Option<HostMetadata>;`
- `pub fn set_host_metadata<HostMetadata: ::wasm_bindgen::JsGeneric>(this: &IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata>, val: &HostMetadata);` → `pub fn set_host_metadata<HostMetadata>(this: &IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata>, val: HostMetadata);`
- `impl<HostMetadata: ::wasm_bindgen::JsGeneric> IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata>` → `impl<HostMetadata: ::wasm_bindgen::convert::IntoWasmAbi> IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata>`
- `pub type IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `IncomingRequestCfPropertiesCloudflareForSaaSEnterpriseBuilder`

- `pub fn host_metadata(self, val: &HostMetadata) -> Self;` → `pub fn host_metadata(self, val: HostMetadata) -> Self;`
- `impl<HostMetadata: ::wasm_bindgen::JsGeneric> From<IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata>> for IncomingRequestCfPropertiesCloudflareForSaaSEnterpriseBuilder<HostMetadata>` → `impl<HostMetadata: ::wasm_bindgen::convert::IntoWasmAbi> From<IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata>> for IncomingRequestCfPropertiesCloudflareForSaaSEnterpriseBuilder<HostMetadata>`
- `impl<HostMetadata: ::wasm_bindgen::JsGeneric> IncomingRequestCfPropertiesCloudflareForSaaSEnterpriseBuilder<HostMetadata>` → `impl<HostMetadata: ::wasm_bindgen::convert::IntoWasmAbi> IncomingRequestCfPropertiesCloudflareForSaaSEnterpriseBuilder<HostMetadata>`
- `pub struct IncomingRequestCfPropertiesCloudflareForSaaSEnterpriseBuilder<HostMetadata: ::wasm_bindgen::JsGeneric> { inner: IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata> }` → `pub struct IncomingRequestCfPropertiesCloudflareForSaaSEnterpriseBuilder<HostMetadata> { inner: IncomingRequestCfPropertiesCloudflareForSaaSEnterprise<HostMetadata> }`

### `IncomingRequestCfPropertiesExportedAuthenticatorMetadata`

- added → `pub fn client_finished_js_string(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata) -> JsString;`
- added → `pub fn client_handshake_js_string(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata) -> JsString;`
- `pub fn new(client_handshake: &str, server_handshake: &str, client_finished: &str, server_finished: &str) -> IncomingRequestCfPropertiesExportedAuthenticatorMetadata;` → `pub fn new(client_handshake: impl ::wasm_bindgen::JsStringLike, server_handshake: impl ::wasm_bindgen::JsStringLike, client_finished: impl ::wasm_bindgen::JsStringLike, server_finished: impl ::wasm_bindgen::JsStringLike) -> IncomingRequestCfPropertiesExportedAuthenticatorMetadata;`
- added → `pub fn server_finished_js_string(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata) -> JsString;`
- added → `pub fn server_handshake_js_string(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata) -> JsString;`
- `pub fn set_client_finished(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata, val: &str);` → `pub fn set_client_finished(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_client_handshake(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata, val: &str);` → `pub fn set_client_handshake(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_server_finished(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata, val: &str);` → `pub fn set_server_finished(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_server_handshake(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata, val: &str);` → `pub fn set_server_handshake(this: &IncomingRequestCfPropertiesExportedAuthenticatorMetadata, val: impl ::wasm_bindgen::JsStringLike);`

### `IncomingRequestCfPropertiesGeographicInformation`

- added → `pub fn city_js_string(this: &IncomingRequestCfPropertiesGeographicInformation) -> Option<JsString>;`
- added → `pub fn is_eu_country_js_string(this: &IncomingRequestCfPropertiesGeographicInformation) -> Option<JsString>;`
- added → `pub fn latitude_js_string(this: &IncomingRequestCfPropertiesGeographicInformation) -> Option<JsString>;`
- added → `pub fn longitude_js_string(this: &IncomingRequestCfPropertiesGeographicInformation) -> Option<JsString>;`
- added → `pub fn metro_code_js_string(this: &IncomingRequestCfPropertiesGeographicInformation) -> Option<JsString>;`
- added → `pub fn postal_code_js_string(this: &IncomingRequestCfPropertiesGeographicInformation) -> Option<JsString>;`
- added → `pub fn region_code_js_string(this: &IncomingRequestCfPropertiesGeographicInformation) -> Option<JsString>;`
- added → `pub fn region_js_string(this: &IncomingRequestCfPropertiesGeographicInformation) -> Option<JsString>;`
- `pub fn set_city(this: &IncomingRequestCfPropertiesGeographicInformation, val: &str);` → `pub fn set_city(this: &IncomingRequestCfPropertiesGeographicInformation, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_is_eu_country(this: &IncomingRequestCfPropertiesGeographicInformation, val: &str);` → `pub fn set_is_eu_country(this: &IncomingRequestCfPropertiesGeographicInformation, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_latitude(this: &IncomingRequestCfPropertiesGeographicInformation, val: &str);` → `pub fn set_latitude(this: &IncomingRequestCfPropertiesGeographicInformation, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_longitude(this: &IncomingRequestCfPropertiesGeographicInformation, val: &str);` → `pub fn set_longitude(this: &IncomingRequestCfPropertiesGeographicInformation, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_metro_code(this: &IncomingRequestCfPropertiesGeographicInformation, val: &str);` → `pub fn set_metro_code(this: &IncomingRequestCfPropertiesGeographicInformation, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_postal_code(this: &IncomingRequestCfPropertiesGeographicInformation, val: &str);` → `pub fn set_postal_code(this: &IncomingRequestCfPropertiesGeographicInformation, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_region(this: &IncomingRequestCfPropertiesGeographicInformation, val: &str);` → `pub fn set_region(this: &IncomingRequestCfPropertiesGeographicInformation, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_region_code(this: &IncomingRequestCfPropertiesGeographicInformation, val: &str);` → `pub fn set_region_code(this: &IncomingRequestCfPropertiesGeographicInformation, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_timezone(this: &IncomingRequestCfPropertiesGeographicInformation, val: &str);` → `pub fn set_timezone(this: &IncomingRequestCfPropertiesGeographicInformation, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn timezone_js_string(this: &IncomingRequestCfPropertiesGeographicInformation) -> Option<JsString>;`

### `IncomingRequestCfPropertiesGeographicInformationBuilder`

- `pub fn city(self, val: &str) -> Self;` → `pub fn city(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn is_eu_country(self, val: &str) -> Self;` → `pub fn is_eu_country(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn latitude(self, val: &str) -> Self;` → `pub fn latitude(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn longitude(self, val: &str) -> Self;` → `pub fn longitude(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn metro_code(self, val: &str) -> Self;` → `pub fn metro_code(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn postal_code(self, val: &str) -> Self;` → `pub fn postal_code(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn region(self, val: &str) -> Self;` → `pub fn region(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn region_code(self, val: &str) -> Self;` → `pub fn region_code(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn timezone(self, val: &str) -> Self;` → `pub fn timezone(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `IncomingRequestCfPropertiesTLSClientAuth`

- added → `pub fn cert_chain_rfc9440_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_fingerprint_sha1_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_fingerprint_sha256_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_issuer_dn_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_issuer_dn_legacy_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_issuer_dnrfc2253_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_issuer_serial_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_issuer_ski_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_not_after_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_not_before_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_presented_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_rfc9440_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_serial_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_ski_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_subject_dn_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_subject_dn_legacy_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- added → `pub fn cert_subject_dnrfc2253_js_string(this: &IncomingRequestCfPropertiesTLSClientAuth) -> JsString;`
- `pub fn new_0(cert_verified: CertVerificationStatus, cert_issuer_dn: &str, cert_subject_dn: &str, cert_issuer_dnrfc2253: &str, cert_subject_dnrfc2253: &str, cert_issuer_dn_legacy: &str, cert_subject_dn_legacy: &str, cert_serial: &str, cert_issuer_serial: &str, cert_ski: &str, cert_issuer_ski: &str, cert_fingerprint_sha1: &str, cert_fingerprint_sha256: &str, cert_not_before: &str, cert_not_after: &str, cert_rfc9440: &str, cert_rfc9440_too_large: bool, cert_chain_rfc9440: &str, cert_chain_rfc9440_too_large: bool) -> IncomingRequestCfPropertiesTLSClientAuth;` → `pub fn new_0(cert_verified: CertVerificationStatus, cert_issuer_dn: impl ::wasm_bindgen::JsStringLike, cert_subject_dn: impl ::wasm_bindgen::JsStringLike, cert_issuer_dnrfc2253: impl ::wasm_bindgen::JsStringLike, cert_subject_dnrfc2253: impl ::wasm_bindgen::JsStringLike, cert_issuer_dn_legacy: impl ::wasm_bindgen::JsStringLike, cert_subject_dn_legacy: impl ::wasm_bindgen::JsStringLike, cert_serial: impl ::wasm_bindgen::JsStringLike, cert_issuer_serial: impl ::wasm_bindgen::JsStringLike, cert_ski: impl ::wasm_bindgen::JsStringLike, cert_issuer_ski: impl ::wasm_bindgen::JsStringLike, cert_fingerprint_sha1: impl ::wasm_bindgen::JsStringLike, cert_fingerprint_sha256: impl ::wasm_bindgen::JsStringLike, cert_not_before: impl ::wasm_bindgen::JsStringLike, cert_not_after: impl ::wasm_bindgen::JsStringLike, cert_rfc9440: impl ::wasm_bindgen::JsStringLike, cert_rfc9440_too_large: bool, cert_chain_rfc9440: impl ::wasm_bindgen::JsStringLike, cert_chain_rfc9440_too_large: bool) -> IncomingRequestCfPropertiesTLSClientAuth;`
- `pub fn new_1(cert_verified: CertVerificationStatus, cert_issuer_dn: &str, cert_subject_dn: &str, cert_issuer_dnrfc2253: &str, cert_subject_dnrfc2253: &str, cert_issuer_dn_legacy: &str, cert_subject_dn_legacy: &str, cert_serial: &str, cert_issuer_serial: &str, cert_ski: &str, cert_issuer_ski: &str, cert_fingerprint_sha1: &str, cert_fingerprint_sha256: &str, cert_not_before: &str, cert_not_after: &str, cert_rfc9440: &str, cert_rfc9440_too_large: bool, cert_chain_rfc9440: &str, cert_chain_rfc9440_too_large: bool) -> IncomingRequestCfPropertiesTLSClientAuth;` → `pub fn new_1(cert_verified: CertVerificationStatus, cert_issuer_dn: impl ::wasm_bindgen::JsStringLike, cert_subject_dn: impl ::wasm_bindgen::JsStringLike, cert_issuer_dnrfc2253: impl ::wasm_bindgen::JsStringLike, cert_subject_dnrfc2253: impl ::wasm_bindgen::JsStringLike, cert_issuer_dn_legacy: impl ::wasm_bindgen::JsStringLike, cert_subject_dn_legacy: impl ::wasm_bindgen::JsStringLike, cert_serial: impl ::wasm_bindgen::JsStringLike, cert_issuer_serial: impl ::wasm_bindgen::JsStringLike, cert_ski: impl ::wasm_bindgen::JsStringLike, cert_issuer_ski: impl ::wasm_bindgen::JsStringLike, cert_fingerprint_sha1: impl ::wasm_bindgen::JsStringLike, cert_fingerprint_sha256: impl ::wasm_bindgen::JsStringLike, cert_not_before: impl ::wasm_bindgen::JsStringLike, cert_not_after: impl ::wasm_bindgen::JsStringLike, cert_rfc9440: impl ::wasm_bindgen::JsStringLike, cert_rfc9440_too_large: bool, cert_chain_rfc9440: impl ::wasm_bindgen::JsStringLike, cert_chain_rfc9440_too_large: bool) -> IncomingRequestCfPropertiesTLSClientAuth;`
- `pub fn set_cert_chain_rfc9440(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_chain_rfc9440(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_fingerprint_sha1(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_fingerprint_sha1(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_fingerprint_sha256(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_fingerprint_sha256(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_dn(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_issuer_dn(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_dn_legacy(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_issuer_dn_legacy(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_dnrfc2253(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_issuer_dnrfc2253(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_serial(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_issuer_serial(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_ski(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_issuer_ski(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_not_after(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_not_after(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_not_before(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_not_before(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_presented(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_presented(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_rfc9440(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_rfc9440(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_serial(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_serial(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_ski(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_ski(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_subject_dn(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_subject_dn(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_subject_dn_legacy(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_subject_dn_legacy(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_subject_dnrfc2253(this: &IncomingRequestCfPropertiesTLSClientAuth, val: &str);` → `pub fn set_cert_subject_dnrfc2253(this: &IncomingRequestCfPropertiesTLSClientAuth, val: impl ::wasm_bindgen::JsStringLike);`

### `IncomingRequestCfPropertiesTLSClientAuthPlaceholder`

- added → `pub fn cert_chain_rfc9440_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_fingerprint_sha1_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_fingerprint_sha256_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_issuer_dn_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_issuer_dn_legacy_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_issuer_dnrfc2253_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_issuer_serial_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_issuer_ski_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_not_after_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_not_before_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_presented_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_revoked_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_rfc9440_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_serial_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_ski_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_subject_dn_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_subject_dn_legacy_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_subject_dnrfc2253_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- added → `pub fn cert_verified_js_string(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder) -> JsString;`
- `pub fn set_cert_chain_rfc9440(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_chain_rfc9440(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_fingerprint_sha1(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_fingerprint_sha1(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_fingerprint_sha256(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_fingerprint_sha256(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_dn(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_issuer_dn(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_dn_legacy(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_issuer_dn_legacy(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_dnrfc2253(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_issuer_dnrfc2253(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_serial(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_issuer_serial(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_issuer_ski(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_issuer_ski(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_not_after(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_not_after(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_not_before(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_not_before(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_presented(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_presented(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_revoked(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_revoked(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_rfc9440(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_rfc9440(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_serial(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_serial(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_ski(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_ski(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_subject_dn(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_subject_dn(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_subject_dn_legacy(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_subject_dn_legacy(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_subject_dnrfc2253(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_subject_dnrfc2253(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cert_verified(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: &str);` → `pub fn set_cert_verified(this: &IncomingRequestCfPropertiesTLSClientAuthPlaceholder, val: impl ::wasm_bindgen::JsStringLike);`

### `JsonWebKey`

- added → `pub fn alg_js_string(this: &JsonWebKey) -> Option<JsString>;`
- `pub fn builder(kty: &str) -> JsonWebKeyBuilder;` → `pub fn builder(kty: impl ::wasm_bindgen::JsStringLike) -> JsonWebKeyBuilder;`
- added → `pub fn crv_js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn d_js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn dp_js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn dq_js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn e_js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn k_js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn key_ops_js_string(this: &JsonWebKey) -> Option<Array<JsString>>;`
- added → `pub fn kty_js_string(this: &JsonWebKey) -> JsString;`
- added → `pub fn n_js_string(this: &JsonWebKey) -> Option<JsString>;`
- `pub fn new(kty: &str) -> JsonWebKey;` → `pub fn new(kty: impl ::wasm_bindgen::JsStringLike) -> JsonWebKey;`
- added → `pub fn p_js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn q_js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn qi_js_string(this: &JsonWebKey) -> Option<JsString>;`
- `pub fn set_alg(this: &JsonWebKey, val: &str);` → `pub fn set_alg(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_crv(this: &JsonWebKey, val: &str);` → `pub fn set_crv(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_d(this: &JsonWebKey, val: &str);` → `pub fn set_d(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_dp(this: &JsonWebKey, val: &str);` → `pub fn set_dp(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_dq(this: &JsonWebKey, val: &str);` → `pub fn set_dq(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_e(this: &JsonWebKey, val: &str);` → `pub fn set_e(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_k(this: &JsonWebKey, val: &str);` → `pub fn set_k(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_key_ops(this: &JsonWebKey, val: &[String]);` → `pub fn set_key_ops(this: &JsonWebKey, val: &[impl ::wasm_bindgen::JsStringLike]);`
- `pub fn set_kty(this: &JsonWebKey, val: &str);` → `pub fn set_kty(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_n(this: &JsonWebKey, val: &str);` → `pub fn set_n(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_p(this: &JsonWebKey, val: &str);` → `pub fn set_p(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_q(this: &JsonWebKey, val: &str);` → `pub fn set_q(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_qi(this: &JsonWebKey, val: &str);` → `pub fn set_qi(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_use(this: &JsonWebKey, val: &str);` → `pub fn set_use(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_x(this: &JsonWebKey, val: &str);` → `pub fn set_x(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_y(this: &JsonWebKey, val: &str);` → `pub fn set_y(this: &JsonWebKey, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn use__js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn x_js_string(this: &JsonWebKey) -> Option<JsString>;`
- added → `pub fn y_js_string(this: &JsonWebKey) -> Option<JsString>;`

### `JsonWebKeyBuilder`

- `pub fn alg(self, val: &str) -> Self;` → `pub fn alg(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn crv(self, val: &str) -> Self;` → `pub fn crv(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn d(self, val: &str) -> Self;` → `pub fn d(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn dp(self, val: &str) -> Self;` → `pub fn dp(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn dq(self, val: &str) -> Self;` → `pub fn dq(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn e(self, val: &str) -> Self;` → `pub fn e(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn k(self, val: &str) -> Self;` → `pub fn k(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn key_ops(self, val: &[String]) -> Self;` → `pub fn key_ops(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`
- `pub fn n(self, val: &str) -> Self;` → `pub fn n(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn p(self, val: &str) -> Self;` → `pub fn p(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn q(self, val: &str) -> Self;` → `pub fn q(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn qi(self, val: &str) -> Self;` → `pub fn qi(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn use_(self, val: &str) -> Self;` → `pub fn use_(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn x(self, val: &str) -> Self;` → `pub fn x(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn y(self, val: &str) -> Self;` → `pub fn y(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `KVNamespace`

- `pub async fn delete(this: &KVNamespace, key: &str) -> Result<(), crate::Error>;` → `pub async fn delete(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn delete_promise(this: &KVNamespace, key: &str) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn delete_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn get(this: &KVNamespace, key: &str) -> Result<Option<String>, crate::Error>;` → `pub async fn get(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike) -> Result<Option<String>, crate::Error>;`
- `pub async fn get_array_buffer(&self, key: &str) -> Result<Option<ArrayBuffer>, crate::Error>;` → `pub async fn get_array_buffer(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Option<ArrayBuffer>, crate::Error>;`
- `async fn get_array_buffer_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Option<ArrayBuffer>, crate::Error>;` → `async fn get_array_buffer_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Option<ArrayBuffer>, crate::Error>;`
- `pub fn get_array_buffer_promise(&self, key: &str) -> Result<Promise<JsNullable<ArrayBuffer>>, crate::Error>;` → `pub fn get_array_buffer_promise(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<ArrayBuffer>>, crate::Error>;`
- `fn get_array_buffer_promise_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Promise<JsNullable<ArrayBuffer>>, crate::Error>;` → `fn get_array_buffer_promise_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<ArrayBuffer>>, crate::Error>;`
- added → `pub async fn get_js_string(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike) -> Result<Option<JsString>, crate::Error>;`
- `pub async fn get_json<ExpectedValue: ::wasm_bindgen::JsGeneric>(&self, key: &str) -> Result<JsNullable<ExpectedValue>, crate::Error>;` → `pub async fn get_json<ExpectedValue>(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<JsNullable<ExpectedValue>, crate::Error>;`
- `async fn get_json_js<ExpectedValue: ::wasm_bindgen::JsGeneric>(this: &KVNamespace, key: &str, r#type: &str) -> Result<JsNullable<ExpectedValue>, crate::Error>;` → `async fn get_json_js<ExpectedValue>(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<JsNullable<ExpectedValue>, crate::Error>;`
- `pub fn get_json_promise<ExpectedValue: ::wasm_bindgen::JsGeneric>(&self, key: &str) -> Result<Promise<JsNullable<ExpectedValue>>, crate::Error>;` → `pub fn get_json_promise<ExpectedValue>(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<ExpectedValue>>, crate::Error>;`
- `fn get_json_promise_js<ExpectedValue: ::wasm_bindgen::JsGeneric>(this: &KVNamespace, key: &str, r#type: &str) -> Result<Promise<JsNullable<ExpectedValue>>, crate::Error>;` → `fn get_json_promise_js<ExpectedValue>(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<ExpectedValue>>, crate::Error>;`
- `pub async fn get_json_with_slice(&self, key: &[String]) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_json_with_slice(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `async fn get_json_with_slice_js(this: &KVNamespace, key: &[String], r#type: &str) -> Result<Map<JsString, JsValue>, crate::Error>;` → `async fn get_json_with_slice_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub fn get_json_with_slice_promise(&self, key: &[String]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_json_with_slice_promise(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `fn get_json_with_slice_promise_js(this: &KVNamespace, key: &[String], r#type: &str) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `fn get_json_with_slice_promise_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `pub fn get_promise(this: &KVNamespace, key: &str) -> Result<Promise<JsNullable<JsString>>, crate::Error>;` → `pub fn get_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<JsString>>, crate::Error>;`
- `pub async fn get_stream(&self, key: &str) -> Result<Option<ReadableStream>, crate::Error>;` → `pub async fn get_stream(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Option<ReadableStream>, crate::Error>;`
- `async fn get_stream_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Option<ReadableStream>, crate::Error>;` → `async fn get_stream_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Option<ReadableStream>, crate::Error>;`
- `pub fn get_stream_promise(&self, key: &str) -> Result<Promise<JsNullable<ReadableStream>>, crate::Error>;` → `pub fn get_stream_promise(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<ReadableStream>>, crate::Error>;`
- `fn get_stream_promise_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Promise<JsNullable<ReadableStream>>, crate::Error>;` → `fn get_stream_promise_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<ReadableStream>>, crate::Error>;`
- `pub async fn get_text(&self, key: &str) -> Result<Option<String>, crate::Error>;` → `pub async fn get_text(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Option<String>, crate::Error>;`
- added → `pub async fn get_text(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Option<JsString>, crate::Error>;`
- `async fn get_text_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Option<String>, crate::Error>;` → `async fn get_text_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Option<String>, crate::Error>;`
- added → `async fn get_text_js_string_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Option<JsString>, crate::Error>;`
- `pub fn get_text_promise(&self, key: &str) -> Result<Promise<JsNullable<JsString>>, crate::Error>;` → `pub fn get_text_promise(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<JsString>>, crate::Error>;`
- `fn get_text_promise_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Promise<JsNullable<JsString>>, crate::Error>;` → `fn get_text_promise_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<JsString>>, crate::Error>;`
- `pub async fn get_text_with_slice(&self, key: &[String]) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `pub async fn get_text_with_slice(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `async fn get_text_with_slice_js(this: &KVNamespace, key: &[String], r#type: &str) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `async fn get_text_with_slice_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `pub fn get_text_with_slice_promise(&self, key: &[String]) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `pub fn get_text_with_slice_promise(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`
- `fn get_text_with_slice_promise_js(this: &KVNamespace, key: &[String], r#type: &str) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `fn get_text_with_slice_promise_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`
- `pub async fn get_with_key_slice_and_options_kv_namespace_get_options(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<JsString>) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `pub async fn get_with_key_slice_and_options_kv_namespace_get_options(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<String>) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `pub fn get_with_key_slice_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<JsString>) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `pub fn get_with_key_slice_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<String>) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`
- `pub async fn get_with_key_str_and_options_kv_namespace_get_options(this: &KVNamespace, key: &str, options: &KVNamespaceGetOptions<JsString>) -> Result<Option<String>, crate::Error>;` → `pub async fn get_with_key_str_and_options_kv_namespace_get_options(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<String>) -> Result<Option<String>, crate::Error>;`
- added → `pub async fn get_with_key_str_and_options_kv_namespace_get_options_js_string(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<String>) -> Result<Option<JsString>, crate::Error>;`
- `pub fn get_with_key_str_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &str, options: &KVNamespaceGetOptions<JsString>) -> Result<Promise<JsNullable<JsString>>, crate::Error>;` → `pub fn get_with_key_str_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<String>) -> Result<Promise<JsNullable<JsString>>, crate::Error>;`
- `pub async fn get_with_metadata(this: &KVNamespace, key: &str) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_metadata(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<String, JsValue>, crate::Error>;`
- `pub async fn get_with_metadata_array_buffer(&self, key: &str) -> Result<KVNamespaceGetWithMetadataResult<ArrayBuffer, JsValue>, crate::Error>;` → `pub async fn get_with_metadata_array_buffer(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<ArrayBuffer, JsValue>, crate::Error>;`
- `async fn get_with_metadata_array_buffer_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<KVNamespaceGetWithMetadataResult<ArrayBuffer, JsValue>, crate::Error>;` → `async fn get_with_metadata_array_buffer_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<ArrayBuffer, JsValue>, crate::Error>;`
- `pub fn get_with_metadata_array_buffer_promise(&self, key: &str) -> Result<Promise<KVNamespaceGetWithMetadataResult<ArrayBuffer, JsValue>>, crate::Error>;` → `pub fn get_with_metadata_array_buffer_promise(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<KVNamespaceGetWithMetadataResult<ArrayBuffer, JsValue>>, crate::Error>;`
- `fn get_with_metadata_array_buffer_promise_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Promise<KVNamespaceGetWithMetadataResult<ArrayBuffer, JsValue>>, crate::Error>;` → `fn get_with_metadata_array_buffer_promise_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<KVNamespaceGetWithMetadataResult<ArrayBuffer, JsValue>>, crate::Error>;`
- added → `pub async fn get_with_metadata_js_string(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;`
- `pub async fn get_with_metadata_json(&self, key: &str) -> Result<KVNamespaceGetWithMetadataResult<JsValue, JsValue>, crate::Error>;` → `pub async fn get_with_metadata_json(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<JsValue, JsValue>, crate::Error>;`
- `async fn get_with_metadata_json_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<KVNamespaceGetWithMetadataResult<JsValue, JsValue>, crate::Error>;` → `async fn get_with_metadata_json_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<JsValue, JsValue>, crate::Error>;`
- `pub fn get_with_metadata_json_promise(&self, key: &str) -> Result<Promise<KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;` → `pub fn get_with_metadata_json_promise(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;`
- `fn get_with_metadata_json_promise_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Promise<KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;` → `fn get_with_metadata_json_promise_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;`
- `pub async fn get_with_metadata_json_with_slice(&self, key: &[String]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_json_with_slice(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;`
- `async fn get_with_metadata_json_with_slice_js(this: &KVNamespace, key: &[String], r#type: &str) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;` → `async fn get_with_metadata_json_with_slice_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;`
- `pub fn get_with_metadata_json_with_slice_promise(&self, key: &[String]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_json_with_slice_promise(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>>, crate::Error>;`
- `fn get_with_metadata_json_with_slice_promise_js(this: &KVNamespace, key: &[String], r#type: &str) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>>, crate::Error>;` → `fn get_with_metadata_json_with_slice_promise_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>>, crate::Error>;`
- `pub fn get_with_metadata_promise(this: &KVNamespace, key: &str) -> Result<Promise<KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_metadata_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub async fn get_with_metadata_stream(&self, key: &str) -> Result<KVNamespaceGetWithMetadataResult<ReadableStream, JsValue>, crate::Error>;` → `pub async fn get_with_metadata_stream(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<ReadableStream, JsValue>, crate::Error>;`
- `async fn get_with_metadata_stream_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<KVNamespaceGetWithMetadataResult<ReadableStream, JsValue>, crate::Error>;` → `async fn get_with_metadata_stream_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<ReadableStream, JsValue>, crate::Error>;`
- `pub fn get_with_metadata_stream_promise(&self, key: &str) -> Result<Promise<KVNamespaceGetWithMetadataResult<ReadableStream, JsValue>>, crate::Error>;` → `pub fn get_with_metadata_stream_promise(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<KVNamespaceGetWithMetadataResult<ReadableStream, JsValue>>, crate::Error>;`
- `fn get_with_metadata_stream_promise_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Promise<KVNamespaceGetWithMetadataResult<ReadableStream, JsValue>>, crate::Error>;` → `fn get_with_metadata_stream_promise_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<KVNamespaceGetWithMetadataResult<ReadableStream, JsValue>>, crate::Error>;`
- `pub async fn get_with_metadata_text(&self, key: &str) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_metadata_text(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<String, JsValue>, crate::Error>;`
- added → `pub async fn get_with_metadata_text(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;`
- `async fn get_with_metadata_text_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;` → `async fn get_with_metadata_text_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<String, JsValue>, crate::Error>;`
- added → `async fn get_with_metadata_text_js_string_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;`
- `pub fn get_with_metadata_text_promise(&self, key: &str) -> Result<Promise<KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_metadata_text_promise(&self, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `fn get_with_metadata_text_promise_js(this: &KVNamespace, key: &str, r#type: &str) -> Result<Promise<KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `fn get_with_metadata_text_promise_js(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub async fn get_with_metadata_text_with_slice(&self, key: &[String]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_text_with_slice(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `async fn get_with_metadata_text_with_slice_js(this: &KVNamespace, key: &[String], r#type: &str) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `async fn get_with_metadata_text_with_slice_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub fn get_with_metadata_text_with_slice_promise(&self, key: &[String]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_text_with_slice_promise(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `fn get_with_metadata_text_with_slice_promise_js(this: &KVNamespace, key: &[String], r#type: &str) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>>, crate::Error>;` → `fn get_with_metadata_text_with_slice_promise_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `pub async fn get_with_metadata_with_key_slice_and_options_kv_namespace_get_options(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<JsString>) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_with_key_slice_and_options_kv_namespace_get_options(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<String>) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub fn get_with_metadata_with_key_slice_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<JsString>) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_with_key_slice_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<String>) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `pub async fn get_with_metadata_with_key_str_and_options_kv_namespace_get_options(this: &KVNamespace, key: &str, options: &KVNamespaceGetOptions<JsString>) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_metadata_with_key_str_and_options_kv_namespace_get_options(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<String>) -> Result<KVNamespaceGetWithMetadataResult<String, JsValue>, crate::Error>;`
- added → `pub async fn get_with_metadata_with_key_str_and_options_kv_namespace_get_options_js_string(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<String>) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;`
- `pub fn get_with_metadata_with_key_str_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &str, options: &KVNamespaceGetOptions<JsString>) -> Result<Promise<KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_metadata_with_key_str_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<String>) -> Result<Promise<KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub async fn get_with_metadata_with_slice(this: &KVNamespace, key: &[String]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_with_slice(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub async fn get_with_metadata_with_slice_and_kv_namespace_get_options(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<Undefined>) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_with_slice_and_kv_namespace_get_options(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<Undefined>) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub fn get_with_metadata_with_slice_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_with_slice_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `pub fn get_with_metadata_with_slice_promise(this: &KVNamespace, key: &[String]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsString, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_with_slice_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `pub async fn get_with_metadata_with_str_and_kv_namespace_get_options(this: &KVNamespace, key: &str, options: &KVNamespaceGetOptions<Undefined>) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_metadata_with_str_and_kv_namespace_get_options(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<Undefined>) -> Result<KVNamespaceGetWithMetadataResult<String, JsValue>, crate::Error>;`
- added → `pub async fn get_with_metadata_with_str_and_kv_namespace_get_options_js_string(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<Undefined>) -> Result<KVNamespaceGetWithMetadataResult<JsString, JsValue>, crate::Error>;`
- `pub fn get_with_metadata_with_str_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &str, options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<KVNamespaceGetWithMetadataResult<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_metadata_with_str_and_kv_namespace_get_options_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub async fn get_with_slice(this: &KVNamespace, key: &[String]) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `pub async fn get_with_slice(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `pub async fn get_with_slice_and_kv_namespace_get_options(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<Undefined>) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `pub async fn get_with_slice_and_kv_namespace_get_options(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<Undefined>) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `pub fn get_with_slice_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `pub fn get_with_slice_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`
- `pub fn get_with_slice_promise(this: &KVNamespace, key: &[String]) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `pub fn get_with_slice_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`
- `pub async fn get_with_str_and_kv_namespace_get_options(this: &KVNamespace, key: &str, options: &KVNamespaceGetOptions<Undefined>) -> Result<Option<String>, crate::Error>;` → `pub async fn get_with_str_and_kv_namespace_get_options(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<Undefined>) -> Result<Option<String>, crate::Error>;`
- added → `pub async fn get_with_str_and_kv_namespace_get_options_js_string(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<Undefined>) -> Result<Option<JsString>, crate::Error>;`
- `pub fn get_with_str_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &str, options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<JsNullable<JsString>>, crate::Error>;` → `pub fn get_with_str_and_kv_namespace_get_options_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<JsNullable<JsString>>, crate::Error>;`
- added → `pub async fn list_js_string(this: &KVNamespace) -> Result<KVNamespaceListResult<JsValue, JsString>, crate::Error>;`
- added → `pub async fn list_with_options_js_string(this: &KVNamespace, options: &KVNamespaceListOptions) -> Result<KVNamespaceListResult<JsValue, JsString>, crate::Error>;`
- `pub async fn put(this: &KVNamespace, key: &str, value: &str) -> Result<(), crate::Error>;` → `pub async fn put(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn put_promise(this: &KVNamespace, key: &str, value: &str) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn put_with_array_buffer(this: &KVNamespace, key: &str, value: &ArrayBuffer) -> Result<(), crate::Error>;` → `pub async fn put_with_array_buffer(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &ArrayBuffer) -> Result<(), crate::Error>;`
- `pub async fn put_with_array_buffer_and_options(this: &KVNamespace, key: &str, value: &ArrayBuffer, options: &KVNamespacePutOptions) -> Result<(), crate::Error>;` → `pub async fn put_with_array_buffer_and_options(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &ArrayBuffer, options: &KVNamespacePutOptions) -> Result<(), crate::Error>;`
- `pub fn put_with_array_buffer_and_options_promise(this: &KVNamespace, key: &str, value: &ArrayBuffer, options: &KVNamespacePutOptions) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_array_buffer_and_options_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &ArrayBuffer, options: &KVNamespacePutOptions) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn put_with_array_buffer_promise(this: &KVNamespace, key: &str, value: &ArrayBuffer) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_array_buffer_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &ArrayBuffer) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn put_with_readable_stream(this: &KVNamespace, key: &str, value: &ReadableStream) -> Result<(), crate::Error>;` → `pub async fn put_with_readable_stream(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &ReadableStream) -> Result<(), crate::Error>;`
- `pub async fn put_with_readable_stream_and_options(this: &KVNamespace, key: &str, value: &ReadableStream, options: &KVNamespacePutOptions) -> Result<(), crate::Error>;` → `pub async fn put_with_readable_stream_and_options(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &ReadableStream, options: &KVNamespacePutOptions) -> Result<(), crate::Error>;`
- `pub fn put_with_readable_stream_and_options_promise(this: &KVNamespace, key: &str, value: &ReadableStream, options: &KVNamespacePutOptions) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_readable_stream_and_options_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &ReadableStream, options: &KVNamespacePutOptions) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn put_with_readable_stream_promise(this: &KVNamespace, key: &str, value: &ReadableStream) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_readable_stream_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &ReadableStream) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn put_with_str_and_options(this: &KVNamespace, key: &str, value: &str, options: &KVNamespacePutOptions) -> Result<(), crate::Error>;` → `pub async fn put_with_str_and_options(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike, options: &KVNamespacePutOptions) -> Result<(), crate::Error>;`
- `pub fn put_with_str_and_options_promise(this: &KVNamespace, key: &str, value: &str, options: &KVNamespacePutOptions) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_str_and_options_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike, options: &KVNamespacePutOptions) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn put_with_typed_array(this: &KVNamespace, key: &str, value: &Object) -> Result<(), crate::Error>;` → `pub async fn put_with_typed_array(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &Object) -> Result<(), crate::Error>;`
- `pub async fn put_with_typed_array_and_options(this: &KVNamespace, key: &str, value: &Object, options: &KVNamespacePutOptions) -> Result<(), crate::Error>;` → `pub async fn put_with_typed_array_and_options(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &Object, options: &KVNamespacePutOptions) -> Result<(), crate::Error>;`
- `pub fn put_with_typed_array_and_options_promise(this: &KVNamespace, key: &str, value: &Object, options: &KVNamespacePutOptions) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_typed_array_and_options_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &Object, options: &KVNamespacePutOptions) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn put_with_typed_array_promise(this: &KVNamespace, key: &str, value: &Object) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn put_with_typed_array_promise(this: &KVNamespace, key: impl ::wasm_bindgen::JsStringLike, value: &Object) -> Result<Promise<Undefined>, crate::Error>;`

### `KVNamespaceGetOptions`

- `pub fn builder(r#type: &Type) -> KVNamespaceGetOptionsBuilder<Type>;` → `pub fn builder(r#type: Type) -> KVNamespaceGetOptionsBuilder<Type>;`
- `fn cache_ttl_js<Type: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetOptions<Type>) -> Option<f64>;` → `fn cache_ttl_js<Type>(this: &KVNamespaceGetOptions<Type>) -> Option<f64>;`
- `pub fn new(r#type: &Type) -> KVNamespaceGetOptions<Type>;` → `pub fn new(r#type: Type) -> KVNamespaceGetOptions<Type>;`
- `fn set_cache_ttl_js<Type: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetOptions<Type>, val: f64);` → `fn set_cache_ttl_js<Type>(this: &KVNamespaceGetOptions<Type>, val: f64);`
- `pub fn set_type<Type: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetOptions<Type>, val: &Type);` → `pub fn set_type<Type>(this: &KVNamespaceGetOptions<Type>, val: Type);`
- `pub fn type_<Type: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetOptions<Type>) -> Type;` → `pub fn type_<Type>(this: &KVNamespaceGetOptions<Type>) -> Type;`
- `impl<Type: ::wasm_bindgen::JsGeneric> KVNamespaceGetOptions<Type>` → `impl<Type> KVNamespaceGetOptions<Type>`
- `impl<Type: ::wasm_bindgen::JsGeneric> KVNamespaceGetOptions<Type>` → `impl<Type: ::wasm_bindgen::convert::IntoWasmAbi> KVNamespaceGetOptions<Type>`
- `pub type KVNamespaceGetOptions<Type: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type KVNamespaceGetOptions<Type = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `KVNamespaceGetOptionsBuilder`

- `impl<Type: ::wasm_bindgen::JsGeneric> From<KVNamespaceGetOptions<Type>> for KVNamespaceGetOptionsBuilder<Type>` → `impl<Type: ::wasm_bindgen::convert::IntoWasmAbi> From<KVNamespaceGetOptions<Type>> for KVNamespaceGetOptionsBuilder<Type>`
- `impl<Type: ::wasm_bindgen::JsGeneric> KVNamespaceGetOptionsBuilder<Type>` → `impl<Type: ::wasm_bindgen::convert::IntoWasmAbi> KVNamespaceGetOptionsBuilder<Type>`
- `pub struct KVNamespaceGetOptionsBuilder<Type: ::wasm_bindgen::JsGeneric> { inner: KVNamespaceGetOptions<Type> }` → `pub struct KVNamespaceGetOptionsBuilder<Type> { inner: KVNamespaceGetOptions<Type> }`

### `KVNamespaceGetWithMetadataResult`

- `pub fn cache_status<Value: ::wasm_bindgen::JsGeneric, Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>) -> Option<String>;` → `pub fn cache_status<Value, Metadata>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>) -> Option<String>;`
- added → `pub fn cache_status_js_string<Value, Metadata>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>) -> Option<JsString>;`
- `pub fn metadata<Value: ::wasm_bindgen::JsGeneric, Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>) -> Option<Metadata>;` → `pub fn metadata<Value, Metadata>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>) -> Option<Metadata>;`
- `pub fn set_cache_status<Value: ::wasm_bindgen::JsGeneric, Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>, val: &JsNullable<JsString>);` → `pub fn set_cache_status<Value, Metadata>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>, val: &JsNullable<JsString>);`
- `pub fn set_metadata<Value: ::wasm_bindgen::JsGeneric, Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>, val: JsNullable<Metadata>);` → `pub fn set_metadata<Value, Metadata>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>, val: JsNullable<Metadata>);`
- `pub fn set_value<Value: ::wasm_bindgen::JsGeneric, Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>, val: JsNullable<Value>);` → `pub fn set_value<Value, Metadata>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>, val: JsNullable<Value>);`
- `pub fn value<Value: ::wasm_bindgen::JsGeneric, Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>) -> Option<Value>;` → `pub fn value<Value, Metadata>(this: &KVNamespaceGetWithMetadataResult<Value, Metadata>) -> Option<Value>;`
- `impl<Value: ::wasm_bindgen::JsGeneric, Metadata: ::wasm_bindgen::JsGeneric> KVNamespaceGetWithMetadataResult<Value, Metadata>` → `impl<Value: ::wasm_bindgen::convert::IntoWasmAbi, Metadata: ::wasm_bindgen::convert::IntoWasmAbi> KVNamespaceGetWithMetadataResult<Value, Metadata>`
- `pub type KVNamespaceGetWithMetadataResult<Value: ::wasm_bindgen::JsGeneric = JsValue, Metadata: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type KVNamespaceGetWithMetadataResult<Value = JsValue, Metadata = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `KVNamespaceListKey`

- `pub fn builder(name: &str) -> KVNamespaceListKeyBuilder<Metadata>;` → `pub fn builder(name: impl ::wasm_bindgen::JsStringLike) -> KVNamespaceListKeyBuilder<Metadata>;`
- `fn expiration_js<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListKey<Metadata>) -> Option<f64>;` → `fn expiration_js<Metadata>(this: &KVNamespaceListKey<Metadata>) -> Option<f64>;`
- `pub fn metadata<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListKey<Metadata>) -> Option<Metadata>;` → `pub fn metadata<Metadata>(this: &KVNamespaceListKey<Metadata>) -> Option<Metadata>;`
- `pub fn name<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListKey<Metadata>) -> String;` → `pub fn name<Metadata>(this: &KVNamespaceListKey<Metadata>) -> String;`
- added → `pub fn name_js_string<Metadata>(this: &KVNamespaceListKey<Metadata>) -> JsString;`
- `pub fn new(name: &str) -> KVNamespaceListKey<Metadata>;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike) -> KVNamespaceListKey<Metadata>;`
- `fn set_expiration_js<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListKey<Metadata>, val: f64);` → `fn set_expiration_js<Metadata>(this: &KVNamespaceListKey<Metadata>, val: f64);`
- `pub fn set_metadata<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListKey<Metadata>, val: &Metadata);` → `pub fn set_metadata<Metadata>(this: &KVNamespaceListKey<Metadata>, val: Metadata);`
- `pub fn set_name<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListKey<Metadata>, val: &str);` → `pub fn set_name<Metadata>(this: &KVNamespaceListKey<Metadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `impl<Metadata: ::wasm_bindgen::JsGeneric> KVNamespaceListKey<Metadata>` → `impl<Metadata> KVNamespaceListKey<Metadata>`
- `impl<Metadata: ::wasm_bindgen::JsGeneric> KVNamespaceListKey<Metadata>` → `impl<Metadata: ::wasm_bindgen::convert::IntoWasmAbi> KVNamespaceListKey<Metadata>`
- `pub type KVNamespaceListKey<Metadata: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type KVNamespaceListKey<Metadata = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `KVNamespaceListKeyBuilder`

- `pub fn metadata(self, val: &Metadata) -> Self;` → `pub fn metadata(self, val: Metadata) -> Self;`
- `impl<Metadata: ::wasm_bindgen::JsGeneric> From<KVNamespaceListKey<Metadata>> for KVNamespaceListKeyBuilder<Metadata>` → `impl<Metadata: ::wasm_bindgen::convert::IntoWasmAbi> From<KVNamespaceListKey<Metadata>> for KVNamespaceListKeyBuilder<Metadata>`
- `impl<Metadata: ::wasm_bindgen::JsGeneric> KVNamespaceListKeyBuilder<Metadata>` → `impl<Metadata: ::wasm_bindgen::convert::IntoWasmAbi> KVNamespaceListKeyBuilder<Metadata>`
- `pub struct KVNamespaceListKeyBuilder<Metadata: ::wasm_bindgen::JsGeneric> { inner: KVNamespaceListKey<Metadata> }` → `pub struct KVNamespaceListKeyBuilder<Metadata> { inner: KVNamespaceListKey<Metadata> }`

### `KVNamespaceListOptions`

- added → `pub fn cursor_js_string(this: &KVNamespaceListOptions) -> Option<JsString>;`
- added → `pub fn prefix_js_string(this: &KVNamespaceListOptions) -> Option<JsString>;`

### `KVNamespaceListResult`

- `pub fn cache_status<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListResult<Metadata>) -> Option<String>;` → `pub fn cache_status<Metadata>(this: &KVNamespaceListResult<Metadata>) -> Option<String>;`
- added → `pub fn cache_status_js_string<Metadata>(this: &KVNamespaceListResult<Metadata>) -> Option<JsString>;`
- `pub fn cursor<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListResult<Metadata>) -> Option<String>;` → `pub fn cursor<Metadata>(this: &KVNamespaceListResult<Metadata>) -> Option<String>;`
- added → `pub fn cursor_js_string<Metadata>(this: &KVNamespaceListResult<Metadata>) -> Option<JsString>;`
- `pub fn keys<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListResult<Metadata>) -> Vec<KVNamespaceListKey<Metadata>>;` → `pub fn keys<Metadata>(this: &KVNamespaceListResult<Metadata>) -> Vec<KVNamespaceListKey<Metadata>>;`
- `pub fn list_complete<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListResult<Metadata>) -> bool;` → `pub fn list_complete<Metadata>(this: &KVNamespaceListResult<Metadata>) -> bool;`
- `pub fn new_false(keys: &[KVNamespaceListKey<Metadata>], cursor: &str, cache_status: &JsNullable<JsString>) -> KVNamespaceListResult<Metadata>;` → `pub fn new_false(keys: &[KVNamespaceListKey<Metadata>], cursor: impl ::wasm_bindgen::JsStringLike, cache_status: &JsNullable<JsString>) -> KVNamespaceListResult<Metadata>;`
- `pub fn set_cache_status<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListResult<Metadata>, val: &JsNullable<JsString>);` → `pub fn set_cache_status<Metadata>(this: &KVNamespaceListResult<Metadata>, val: &JsNullable<JsString>);`
- `pub fn set_cursor<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListResult<Metadata>, val: &str);` → `pub fn set_cursor<Metadata>(this: &KVNamespaceListResult<Metadata>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_keys<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListResult<Metadata>, val: &[KVNamespaceListKey<Metadata>]);` → `pub fn set_keys<Metadata>(this: &KVNamespaceListResult<Metadata>, val: &[KVNamespaceListKey<Metadata>]);`
- `pub fn set_list_complete<Metadata: ::wasm_bindgen::JsGeneric>(this: &KVNamespaceListResult<Metadata>, val: bool);` → `pub fn set_list_complete<Metadata>(this: &KVNamespaceListResult<Metadata>, val: bool);`
- `impl<Metadata: ::wasm_bindgen::JsGeneric> KVNamespaceListResult<Metadata>` → `impl<Metadata: ::wasm_bindgen::convert::IntoWasmAbi> KVNamespaceListResult<Metadata>`
- `pub type KVNamespaceListResult<Metadata: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type KVNamespaceListResult<Metadata = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `KVNamespaceListResultBuilder`

- `pub fn cursor(self, val: &str) -> Self;` → `pub fn cursor(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `impl<Metadata: ::wasm_bindgen::JsGeneric> From<KVNamespaceListResult<Metadata>> for KVNamespaceListResultBuilder<Metadata>` → `impl<Metadata: ::wasm_bindgen::convert::IntoWasmAbi> From<KVNamespaceListResult<Metadata>> for KVNamespaceListResultBuilder<Metadata>`
- `impl<Metadata: ::wasm_bindgen::JsGeneric> KVNamespaceListResultBuilder<Metadata>` → `impl<Metadata: ::wasm_bindgen::convert::IntoWasmAbi> KVNamespaceListResultBuilder<Metadata>`
- `pub struct KVNamespaceListResultBuilder<Metadata: ::wasm_bindgen::JsGeneric> { inner: KVNamespaceListResult<Metadata> }` → `pub struct KVNamespaceListResultBuilder<Metadata> { inner: KVNamespaceListResult<Metadata> }`

### `Message`

- `pub fn ack<Body: ::wasm_bindgen::JsGeneric>(this: &Message<Body>) -> Result<(), crate::Error>;` → `pub fn ack<Body>(this: &Message<Body>) -> Result<(), crate::Error>;`
- `pub fn attempts<Body: ::wasm_bindgen::JsGeneric>(this: &Message<Body>) -> f64;` → `pub fn attempts<Body>(this: &Message<Body>) -> f64;`
- `pub fn body<Body: ::wasm_bindgen::JsGeneric>(this: &Message<Body>) -> Body;` → `pub fn body<Body>(this: &Message<Body>) -> Body;`
- `pub fn id<Body: ::wasm_bindgen::JsGeneric>(this: &Message<Body>) -> String;` → `pub fn id<Body>(this: &Message<Body>) -> String;`
- added → `pub fn id_js_string<Body>(this: &Message<Body>) -> JsString;`
- `pub fn retry<Body: ::wasm_bindgen::JsGeneric>(this: &Message<Body>) -> Result<(), crate::Error>;` → `pub fn retry<Body>(this: &Message<Body>) -> Result<(), crate::Error>;`
- `pub fn retry_with_options<Body: ::wasm_bindgen::JsGeneric>(this: &Message<Body>, options: &QueueRetryOptions) -> Result<(), crate::Error>;` → `pub fn retry_with_options<Body>(this: &Message<Body>, options: &QueueRetryOptions) -> Result<(), crate::Error>;`
- `pub fn timestamp<Body: ::wasm_bindgen::JsGeneric>(this: &Message<Body>) -> Date;` → `pub fn timestamp<Body>(this: &Message<Body>) -> Date;`
- `pub type Message<Body: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type Message<Body = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `MessageBatch`

- `pub fn ack_all<Body: ::wasm_bindgen::JsGeneric>(this: &MessageBatch<Body>) -> Result<(), crate::Error>;` → `pub fn ack_all<Body>(this: &MessageBatch<Body>) -> Result<(), crate::Error>;`
- `pub fn messages<Body: ::wasm_bindgen::JsGeneric>(this: &MessageBatch<Body>) -> Vec<Message<Body>>;` → `pub fn messages<Body>(this: &MessageBatch<Body>) -> Vec<Message<Body>>;`
- `pub fn metadata<Body: ::wasm_bindgen::JsGeneric>(this: &MessageBatch<Body>) -> MessageBatchMetadata;` → `pub fn metadata<Body>(this: &MessageBatch<Body>) -> MessageBatchMetadata;`
- `pub fn queue<Body: ::wasm_bindgen::JsGeneric>(this: &MessageBatch<Body>) -> String;` → `pub fn queue<Body>(this: &MessageBatch<Body>) -> String;`
- added → `pub fn queue_js_string<Body>(this: &MessageBatch<Body>) -> JsString;`
- `pub fn retry_all<Body: ::wasm_bindgen::JsGeneric>(this: &MessageBatch<Body>) -> Result<(), crate::Error>;` → `pub fn retry_all<Body>(this: &MessageBatch<Body>) -> Result<(), crate::Error>;`
- `pub fn retry_all_with_options<Body: ::wasm_bindgen::JsGeneric>(this: &MessageBatch<Body>, options: &QueueRetryOptions) -> Result<(), crate::Error>;` → `pub fn retry_all_with_options<Body>(this: &MessageBatch<Body>, options: &QueueRetryOptions) -> Result<(), crate::Error>;`
- `pub type MessageBatch<Body: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type MessageBatch<Body = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `MessageEvent`

- added → `pub fn last_event_id_js_string(this: &MessageEvent) -> JsString;`
- `pub fn new(r#type: &str, initializer: &MessageEventInit) -> Result<MessageEvent, crate::Error>;` → `pub fn new(r#type: impl ::wasm_bindgen::JsStringLike, initializer: &MessageEventInit) -> Result<MessageEvent, crate::Error>;`
- added → `pub fn origin_js_string(this: &MessageEvent) -> Option<JsString>;`

### `MessageEventInit`

- `pub fn new_with_str(data: &str) -> MessageEventInit;` → `pub fn new_with_str(data: impl ::wasm_bindgen::JsStringLike) -> MessageEventInit;`
- `pub fn set_data_with_str(this: &MessageEventInit, val: &str);` → `pub fn set_data_with_str(this: &MessageEventInit, val: impl ::wasm_bindgen::JsStringLike);`

### `MessageSendRequest`

- `pub fn body<Body: ::wasm_bindgen::JsGeneric>(this: &MessageSendRequest<Body>) -> Body;` → `pub fn body<Body>(this: &MessageSendRequest<Body>) -> Body;`
- `pub fn builder(body: &Body) -> MessageSendRequestBuilder<Body>;` → `pub fn builder(body: Body) -> MessageSendRequestBuilder<Body>;`
- `pub fn content_type<Body: ::wasm_bindgen::JsGeneric>(this: &MessageSendRequest<Body>) -> Option<QueueContentType>;` → `pub fn content_type<Body>(this: &MessageSendRequest<Body>) -> Option<QueueContentType>;`
- `pub fn delay_seconds<Body: ::wasm_bindgen::JsGeneric>(this: &MessageSendRequest<Body>) -> Option<f64>;` → `pub fn delay_seconds<Body>(this: &MessageSendRequest<Body>) -> Option<f64>;`
- `pub fn new(body: &Body) -> MessageSendRequest<Body>;` → `pub fn new(body: Body) -> MessageSendRequest<Body>;`
- `pub fn set_body<Body: ::wasm_bindgen::JsGeneric>(this: &MessageSendRequest<Body>, val: &Body);` → `pub fn set_body<Body>(this: &MessageSendRequest<Body>, val: Body);`
- `pub fn set_content_type<Body: ::wasm_bindgen::JsGeneric>(this: &MessageSendRequest<Body>, val: QueueContentType);` → `pub fn set_content_type<Body>(this: &MessageSendRequest<Body>, val: QueueContentType);`
- `pub fn set_delay_seconds<Body: ::wasm_bindgen::JsGeneric>(this: &MessageSendRequest<Body>, val: f64);` → `pub fn set_delay_seconds<Body>(this: &MessageSendRequest<Body>, val: f64);`
- `impl<Body: ::wasm_bindgen::JsGeneric> MessageSendRequest<Body>` → `impl<Body: ::wasm_bindgen::convert::IntoWasmAbi> MessageSendRequest<Body>`
- `pub type MessageSendRequest<Body: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type MessageSendRequest<Body = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `MessageSendRequestBuilder`

- `impl<Body: ::wasm_bindgen::JsGeneric> From<MessageSendRequest<Body>> for MessageSendRequestBuilder<Body>` → `impl<Body: ::wasm_bindgen::convert::IntoWasmAbi> From<MessageSendRequest<Body>> for MessageSendRequestBuilder<Body>`
- `impl<Body: ::wasm_bindgen::JsGeneric> MessageSendRequestBuilder<Body>` → `impl<Body: ::wasm_bindgen::convert::IntoWasmAbi> MessageSendRequestBuilder<Body>`
- `pub struct MessageSendRequestBuilder<Body: ::wasm_bindgen::JsGeneric> { inner: MessageSendRequest<Body> }` → `pub struct MessageSendRequestBuilder<Body> { inner: MessageSendRequest<Body> }`

### `Performance`

- `pub fn clear_marks_with_name(this: &Performance, name: &str) -> Result<(), crate::Error>;` → `pub fn clear_marks_with_name(this: &Performance, name: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn clear_measures_with_name(this: &Performance, name: &str) -> Result<(), crate::Error>;` → `pub fn clear_measures_with_name(this: &Performance, name: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn get_entries_by_name(this: &Performance, name: &str) -> Result<Vec<PerformanceEntry>, crate::Error>;` → `pub fn get_entries_by_name(this: &Performance, name: impl ::wasm_bindgen::JsStringLike) -> Result<Vec<PerformanceEntry>, crate::Error>;`
- `pub fn get_entries_by_name_with_type(this: &Performance, name: &str, r#type: &str) -> Result<Vec<PerformanceEntry>, crate::Error>;` → `pub fn get_entries_by_name_with_type(this: &Performance, name: impl ::wasm_bindgen::JsStringLike, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Vec<PerformanceEntry>, crate::Error>;`
- `pub fn get_entries_by_type(this: &Performance, r#type: &str) -> Result<Vec<PerformanceEntry>, crate::Error>;` → `pub fn get_entries_by_type(this: &Performance, r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Vec<PerformanceEntry>, crate::Error>;`
- `pub fn mark(this: &Performance, name: &str) -> Result<PerformanceMark, crate::Error>;` → `pub fn mark(this: &Performance, name: impl ::wasm_bindgen::JsStringLike) -> Result<PerformanceMark, crate::Error>;`
- `pub fn mark_with_options(this: &Performance, name: &str, options: &PerformanceMarkOptions) -> Result<PerformanceMark, crate::Error>;` → `pub fn mark_with_options(this: &Performance, name: impl ::wasm_bindgen::JsStringLike, options: &PerformanceMarkOptions) -> Result<PerformanceMark, crate::Error>;`
- `pub fn measure(this: &Performance, measure_name: &str) -> Result<PerformanceMeasure, crate::Error>;` → `pub fn measure(this: &Performance, measure_name: impl ::wasm_bindgen::JsStringLike) -> Result<PerformanceMeasure, crate::Error>;`
- `pub fn measure_with_performance_measure_options(this: &Performance, measure_name: &str, measure_options_or_start_mark: &PerformanceMeasureOptions) -> Result<PerformanceMeasure, crate::Error>;` → `pub fn measure_with_performance_measure_options(this: &Performance, measure_name: impl ::wasm_bindgen::JsStringLike, measure_options_or_start_mark: &PerformanceMeasureOptions) -> Result<PerformanceMeasure, crate::Error>;`
- `pub fn measure_with_performance_measure_options_and_maybe_end_mark(this: &Performance, measure_name: &str, measure_options_or_start_mark: &PerformanceMeasureOptions, maybe_end_mark: &str) -> Result<PerformanceMeasure, crate::Error>;` → `pub fn measure_with_performance_measure_options_and_maybe_end_mark(this: &Performance, measure_name: impl ::wasm_bindgen::JsStringLike, measure_options_or_start_mark: &PerformanceMeasureOptions, maybe_end_mark: impl ::wasm_bindgen::JsStringLike) -> Result<PerformanceMeasure, crate::Error>;`
- `pub fn measure_with_str(this: &Performance, measure_name: &str, measure_options_or_start_mark: &str) -> Result<PerformanceMeasure, crate::Error>;` → `pub fn measure_with_str(this: &Performance, measure_name: impl ::wasm_bindgen::JsStringLike, measure_options_or_start_mark: impl ::wasm_bindgen::JsStringLike) -> Result<PerformanceMeasure, crate::Error>;`
- `pub fn measure_with_str_and_maybe_end_mark(this: &Performance, measure_name: &str, measure_options_or_start_mark: &str, maybe_end_mark: &str) -> Result<PerformanceMeasure, crate::Error>;` → `pub fn measure_with_str_and_maybe_end_mark(this: &Performance, measure_name: impl ::wasm_bindgen::JsStringLike, measure_options_or_start_mark: impl ::wasm_bindgen::JsStringLike, maybe_end_mark: impl ::wasm_bindgen::JsStringLike) -> Result<PerformanceMeasure, crate::Error>;`

### `PerformanceEntry`

- added → `pub fn entry_type_js_string(this: &PerformanceEntry) -> JsString;`
- added → `pub fn name_js_string(this: &PerformanceEntry) -> JsString;`

### `Queue`

- `pub async fn metrics<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>) -> Result<QueueMetrics, crate::Error>;` → `pub async fn metrics<Body>(this: &Queue<Body>) -> Result<QueueMetrics, crate::Error>;`
- `pub fn metrics_promise<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>) -> Result<Promise<QueueMetrics>, crate::Error>;` → `pub fn metrics_promise<Body>(this: &Queue<Body>) -> Result<Promise<QueueMetrics>, crate::Error>;`
- `pub async fn send<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>, message: &Body) -> Result<QueueSendResponse, crate::Error>;` → `pub async fn send<Body>(this: &Queue<Body>, message: Body) -> Result<QueueSendResponse, crate::Error>;`
- `pub async fn send_batch<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>, messages: &JsValue) -> Result<QueueSendBatchResponse, crate::Error>;` → `pub async fn send_batch<Body>(this: &Queue<Body>, messages: &JsValue) -> Result<QueueSendBatchResponse, crate::Error>;`
- `pub fn send_batch_promise<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>, messages: &JsValue) -> Result<Promise<QueueSendBatchResponse>, crate::Error>;` → `pub fn send_batch_promise<Body>(this: &Queue<Body>, messages: &JsValue) -> Result<Promise<QueueSendBatchResponse>, crate::Error>;`
- `pub async fn send_batch_with_options<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>, messages: &JsValue, options: &QueueSendBatchOptions) -> Result<QueueSendBatchResponse, crate::Error>;` → `pub async fn send_batch_with_options<Body>(this: &Queue<Body>, messages: &JsValue, options: &QueueSendBatchOptions) -> Result<QueueSendBatchResponse, crate::Error>;`
- `pub fn send_batch_with_options_promise<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>, messages: &JsValue, options: &QueueSendBatchOptions) -> Result<Promise<QueueSendBatchResponse>, crate::Error>;` → `pub fn send_batch_with_options_promise<Body>(this: &Queue<Body>, messages: &JsValue, options: &QueueSendBatchOptions) -> Result<Promise<QueueSendBatchResponse>, crate::Error>;`
- `pub fn send_promise<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>, message: &Body) -> Result<Promise<QueueSendResponse>, crate::Error>;` → `pub fn send_promise<Body>(this: &Queue<Body>, message: Body) -> Result<Promise<QueueSendResponse>, crate::Error>;`
- `pub async fn send_with_options<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>, message: &Body, options: &QueueSendOptions) -> Result<QueueSendResponse, crate::Error>;` → `pub async fn send_with_options<Body>(this: &Queue<Body>, message: Body, options: &QueueSendOptions) -> Result<QueueSendResponse, crate::Error>;`
- `pub fn send_with_options_promise<Body: ::wasm_bindgen::JsGeneric>(this: &Queue<Body>, message: &Body, options: &QueueSendOptions) -> Result<Promise<QueueSendResponse>, crate::Error>;` → `pub fn send_with_options_promise<Body>(this: &Queue<Body>, message: Body, options: &QueueSendOptions) -> Result<Promise<QueueSendResponse>, crate::Error>;`
- `pub type Queue<Body: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type Queue<Body = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `QueuingStrategy`

- `pub fn high_water_mark<T: ::wasm_bindgen::JsGeneric>(this: &QueuingStrategy<T>) -> Option<BytesWrittenKind>;` → `pub fn high_water_mark<T>(this: &QueuingStrategy<T>) -> Option<BytesWrittenKind>;`
- `pub fn set_high_water_mark<T: ::wasm_bindgen::JsGeneric>(this: &QueuingStrategy<T>, val: f64);` → `pub fn set_high_water_mark<T>(this: &QueuingStrategy<T>, val: impl ::js_sys::JsBigIntOrNumberLike);`
- `pub fn set_high_water_mark_with_big_int<T: ::wasm_bindgen::JsGeneric>(this: &QueuingStrategy<T>, val: &BigInt);` → removed
- `pub fn set_size<T: ::wasm_bindgen::JsGeneric>(this: &QueuingStrategy<T>, val: &Function<fn(T) -> JsValue>);` → `pub fn set_size<T>(this: &QueuingStrategy<T>, val: &Function<fn(T) -> JsValue>);`
- `pub fn size<T: ::wasm_bindgen::JsGeneric>(this: &QueuingStrategy<T>) -> Option<Function<fn(T) -> JsValue>>;` → `pub fn size<T>(this: &QueuingStrategy<T>) -> Option<Function<fn(T) -> JsValue>>;`
- `impl<T: ::wasm_bindgen::JsGeneric> QueuingStrategy<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> QueuingStrategy<T>`
- `pub type QueuingStrategy<T: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type QueuingStrategy<T = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `QueuingStrategyBuilder`

- `pub fn high_water_mark(self, val: f64) -> Self;` → `pub fn high_water_mark(self, val: impl ::js_sys::JsBigIntOrNumberLike) -> Self;`
- `pub fn high_water_mark_with_big_int(self, val: &BigInt) -> Self;` → removed
- `impl<T: ::wasm_bindgen::JsGeneric> From<QueuingStrategy<T>> for QueuingStrategyBuilder<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> From<QueuingStrategy<T>> for QueuingStrategyBuilder<T>`
- `impl<T: ::wasm_bindgen::JsGeneric> QueuingStrategyBuilder<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> QueuingStrategyBuilder<T>`
- `pub struct QueuingStrategyBuilder<T: ::wasm_bindgen::JsGeneric> { inner: QueuingStrategy<T> }` → `pub struct QueuingStrategyBuilder<T> { inner: QueuingStrategy<T> }`

### `R2Bucket`

- `pub async fn create_multipart_upload(this: &R2Bucket, key: &str) -> Result<R2MultipartUpload, crate::Error>;` → `pub async fn create_multipart_upload(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike) -> Result<R2MultipartUpload, crate::Error>;`
- `pub fn create_multipart_upload_promise(this: &R2Bucket, key: &str) -> Result<Promise<R2MultipartUpload>, crate::Error>;` → `pub fn create_multipart_upload_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<R2MultipartUpload>, crate::Error>;`
- `pub async fn create_multipart_upload_with_options(this: &R2Bucket, key: &str, options: &R2MultipartOptions) -> Result<R2MultipartUpload, crate::Error>;` → `pub async fn create_multipart_upload_with_options(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, options: &R2MultipartOptions) -> Result<R2MultipartUpload, crate::Error>;`
- `pub fn create_multipart_upload_with_options_promise(this: &R2Bucket, key: &str, options: &R2MultipartOptions) -> Result<Promise<R2MultipartUpload>, crate::Error>;` → `pub fn create_multipart_upload_with_options_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, options: &R2MultipartOptions) -> Result<Promise<R2MultipartUpload>, crate::Error>;`
- `pub async fn delete(this: &R2Bucket, keys: &str) -> Result<(), crate::Error>;` → `pub async fn delete(this: &R2Bucket, keys: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn delete_promise(this: &R2Bucket, keys: &str) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn delete_promise(this: &R2Bucket, keys: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn delete_with_slice(this: &R2Bucket, keys: &[String]) -> Result<(), crate::Error>;` → `pub async fn delete_with_slice(this: &R2Bucket, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<(), crate::Error>;`
- `pub fn delete_with_slice_promise(this: &R2Bucket, keys: &[String]) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn delete_with_slice_promise(this: &R2Bucket, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn get(this: &R2Bucket, key: &str) -> Result<Option<R2ObjectBody>, crate::Error>;` → `pub async fn get(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike) -> Result<Option<R2ObjectBody>, crate::Error>;`
- `pub fn get_promise(this: &R2Bucket, key: &str) -> Result<Promise<JsNullable<R2ObjectBody>>, crate::Error>;` → `pub fn get_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<R2ObjectBody>>, crate::Error>;`
- `pub async fn get_with_r2_bucket_options(this: &R2Bucket, key: &str, options: &R2BucketOptions) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn get_with_r2_bucket_options(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, options: &R2BucketOptions) -> Result<Option<R2Object>, crate::Error>;`
- `pub fn get_with_r2_bucket_options_promise(this: &R2Bucket, key: &str, options: &R2BucketOptions) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn get_with_r2_bucket_options_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, options: &R2BucketOptions) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn get_with_r2_get_options(this: &R2Bucket, key: &str, options: &R2GetOptions) -> Result<Option<R2ObjectBody>, crate::Error>;` → `pub async fn get_with_r2_get_options(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, options: &R2GetOptions) -> Result<Option<R2ObjectBody>, crate::Error>;`
- `pub fn get_with_r2_get_options_promise(this: &R2Bucket, key: &str, options: &R2GetOptions) -> Result<Promise<JsNullable<R2ObjectBody>>, crate::Error>;` → `pub fn get_with_r2_get_options_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, options: &R2GetOptions) -> Result<Promise<JsNullable<R2ObjectBody>>, crate::Error>;`
- `pub async fn head(this: &R2Bucket, key: &str) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn head(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike) -> Result<Option<R2Object>, crate::Error>;`
- `pub fn head_promise(this: &R2Bucket, key: &str) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn head_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put(this: &R2Bucket, key: &str, value: &JsNullable<ReadableStream>) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ReadableStream>) -> Result<Option<R2Object>, crate::Error>;`
- `pub fn put_promise(this: &R2Bucket, key: &str, value: &JsNullable<ReadableStream>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ReadableStream>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put_with_array_buffer(this: &R2Bucket, key: &str, value: &JsNullable<ArrayBuffer>) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put_with_array_buffer(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ArrayBuffer>) -> Result<Option<R2Object>, crate::Error>;`
- `pub async fn put_with_array_buffer_and_r2_bucket_options2(this: &R2Bucket, key: &str, value: &JsNullable<ArrayBuffer>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put_with_array_buffer_and_r2_bucket_options2(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ArrayBuffer>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;`
- `pub fn put_with_array_buffer_and_r2_bucket_options2_promise(this: &R2Bucket, key: &str, value: &JsNullable<ArrayBuffer>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_with_array_buffer_and_r2_bucket_options2_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ArrayBuffer>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put_with_array_buffer_and_r2_put_options(this: &R2Bucket, key: &str, value: &JsNullable<ArrayBuffer>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;` → `pub async fn put_with_array_buffer_and_r2_put_options(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ArrayBuffer>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;`
- `pub fn put_with_array_buffer_and_r2_put_options_promise(this: &R2Bucket, key: &str, value: &JsNullable<ArrayBuffer>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;` → `pub fn put_with_array_buffer_and_r2_put_options_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ArrayBuffer>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;`
- `pub fn put_with_array_buffer_promise(this: &R2Bucket, key: &str, value: &JsNullable<ArrayBuffer>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_with_array_buffer_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ArrayBuffer>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put_with_blob(this: &R2Bucket, key: &str, value: &JsNullable<Blob>) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put_with_blob(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Blob>) -> Result<Option<R2Object>, crate::Error>;`
- `pub async fn put_with_blob_and_r2_bucket_options2(this: &R2Bucket, key: &str, value: &JsNullable<Blob>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put_with_blob_and_r2_bucket_options2(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Blob>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;`
- `pub fn put_with_blob_and_r2_bucket_options2_promise(this: &R2Bucket, key: &str, value: &JsNullable<Blob>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_with_blob_and_r2_bucket_options2_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Blob>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put_with_blob_and_r2_put_options(this: &R2Bucket, key: &str, value: &JsNullable<Blob>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;` → `pub async fn put_with_blob_and_r2_put_options(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Blob>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;`
- `pub fn put_with_blob_and_r2_put_options_promise(this: &R2Bucket, key: &str, value: &JsNullable<Blob>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;` → `pub fn put_with_blob_and_r2_put_options_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Blob>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;`
- `pub fn put_with_blob_promise(this: &R2Bucket, key: &str, value: &JsNullable<Blob>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_with_blob_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Blob>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put_with_readable_stream_and_r2_bucket_options2(this: &R2Bucket, key: &str, value: &JsNullable<ReadableStream>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put_with_readable_stream_and_r2_bucket_options2(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ReadableStream>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;`
- `pub fn put_with_readable_stream_and_r2_bucket_options2_promise(this: &R2Bucket, key: &str, value: &JsNullable<ReadableStream>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_with_readable_stream_and_r2_bucket_options2_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ReadableStream>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put_with_readable_stream_and_r2_put_options(this: &R2Bucket, key: &str, value: &JsNullable<ReadableStream>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;` → `pub async fn put_with_readable_stream_and_r2_put_options(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ReadableStream>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;`
- `pub fn put_with_readable_stream_and_r2_put_options_promise(this: &R2Bucket, key: &str, value: &JsNullable<ReadableStream>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;` → `pub fn put_with_readable_stream_and_r2_put_options_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<ReadableStream>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;`
- `pub async fn put_with_str(this: &R2Bucket, key: &str, value: &JsNullable<JsString>) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put_with_str(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<JsString>) -> Result<Option<R2Object>, crate::Error>;`
- `pub async fn put_with_str_and_r2_bucket_options2(this: &R2Bucket, key: &str, value: &JsNullable<JsString>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put_with_str_and_r2_bucket_options2(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<JsString>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;`
- `pub fn put_with_str_and_r2_bucket_options2_promise(this: &R2Bucket, key: &str, value: &JsNullable<JsString>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_with_str_and_r2_bucket_options2_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<JsString>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put_with_str_and_r2_put_options(this: &R2Bucket, key: &str, value: &JsNullable<JsString>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;` → `pub async fn put_with_str_and_r2_put_options(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<JsString>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;`
- `pub fn put_with_str_and_r2_put_options_promise(this: &R2Bucket, key: &str, value: &JsNullable<JsString>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;` → `pub fn put_with_str_and_r2_put_options_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<JsString>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;`
- `pub fn put_with_str_promise(this: &R2Bucket, key: &str, value: &JsNullable<JsString>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_with_str_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<JsString>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put_with_typed_array(this: &R2Bucket, key: &str, value: &JsNullable<Object>) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put_with_typed_array(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Object>) -> Result<Option<R2Object>, crate::Error>;`
- `pub async fn put_with_typed_array_and_r2_bucket_options2(this: &R2Bucket, key: &str, value: &JsNullable<Object>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;` → `pub async fn put_with_typed_array_and_r2_bucket_options2(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Object>, options: &R2BucketOptions2) -> Result<Option<R2Object>, crate::Error>;`
- `pub fn put_with_typed_array_and_r2_bucket_options2_promise(this: &R2Bucket, key: &str, value: &JsNullable<Object>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_with_typed_array_and_r2_bucket_options2_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Object>, options: &R2BucketOptions2) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub async fn put_with_typed_array_and_r2_put_options(this: &R2Bucket, key: &str, value: &JsNullable<Object>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;` → `pub async fn put_with_typed_array_and_r2_put_options(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Object>, options: &R2PutOptions) -> Result<R2Object, crate::Error>;`
- `pub fn put_with_typed_array_and_r2_put_options_promise(this: &R2Bucket, key: &str, value: &JsNullable<Object>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;` → `pub fn put_with_typed_array_and_r2_put_options_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Object>, options: &R2PutOptions) -> Result<Promise<R2Object>, crate::Error>;`
- `pub fn put_with_typed_array_promise(this: &R2Bucket, key: &str, value: &JsNullable<Object>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;` → `pub fn put_with_typed_array_promise(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, value: &JsNullable<Object>) -> Result<Promise<JsNullable<R2Object>>, crate::Error>;`
- `pub fn resume_multipart_upload(this: &R2Bucket, key: &str, upload_id: &str) -> Result<R2MultipartUpload, crate::Error>;` → `pub fn resume_multipart_upload(this: &R2Bucket, key: impl ::wasm_bindgen::JsStringLike, upload_id: impl ::wasm_bindgen::JsStringLike) -> Result<R2MultipartUpload, crate::Error>;`

### `R2BucketOptions2Builder`

- `pub fn md5_with_str(self, val: &str) -> Self;` → `pub fn md5_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn md5_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn md5_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn sha1_with_str(self, val: &str) -> Self;` → `pub fn sha1_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha1_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn sha1_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn sha256_with_str(self, val: &str) -> Self;` → `pub fn sha256_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha256_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn sha256_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn sha384_with_str(self, val: &str) -> Self;` → `pub fn sha384_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha384_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn sha384_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn sha512_with_str(self, val: &str) -> Self;` → `pub fn sha512_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha512_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn sha512_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn ssec_key_with_str(self, val: &str) -> Self;` → `pub fn ssec_key_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn storage_class(self, val: &str) -> Self;` → `pub fn storage_class(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2BucketOptionsBuilder`

- `pub fn ssec_key_with_str(self, val: &str) -> Self;` → `pub fn ssec_key_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2Conditional`

- added → `pub fn etag_does_not_match_js_string(this: &R2Conditional) -> Option<JsString>;`
- added → `pub fn etag_matches_js_string(this: &R2Conditional) -> Option<JsString>;`
- `pub fn set_etag_does_not_match(this: &R2Conditional, val: &str);` → `pub fn set_etag_does_not_match(this: &R2Conditional, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_etag_matches(this: &R2Conditional, val: &str);` → `pub fn set_etag_matches(this: &R2Conditional, val: impl ::wasm_bindgen::JsStringLike);`

### `R2ConditionalBuilder`

- `pub fn etag_does_not_match(self, val: &str) -> Self;` → `pub fn etag_does_not_match(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn etag_matches(self, val: &str) -> Self;` → `pub fn etag_matches(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2GetOptions`

- `pub fn set_ssec_key_with_str(this: &R2GetOptions, val: &str);` → `pub fn set_ssec_key_with_str(this: &R2GetOptions, val: impl ::wasm_bindgen::JsStringLike);`

### `R2GetOptionsBuilder`

- `pub fn ssec_key_with_str(self, val: &str) -> Self;` → `pub fn ssec_key_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2HTTPMetadata`

- added → `pub fn cache_control_js_string(this: &R2HTTPMetadata) -> Option<JsString>;`
- added → `pub fn content_disposition_js_string(this: &R2HTTPMetadata) -> Option<JsString>;`
- added → `pub fn content_encoding_js_string(this: &R2HTTPMetadata) -> Option<JsString>;`
- added → `pub fn content_language_js_string(this: &R2HTTPMetadata) -> Option<JsString>;`
- added → `pub fn content_type_js_string(this: &R2HTTPMetadata) -> Option<JsString>;`
- `pub fn set_cache_control(this: &R2HTTPMetadata, val: &str);` → `pub fn set_cache_control(this: &R2HTTPMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_content_disposition(this: &R2HTTPMetadata, val: &str);` → `pub fn set_content_disposition(this: &R2HTTPMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_content_encoding(this: &R2HTTPMetadata, val: &str);` → `pub fn set_content_encoding(this: &R2HTTPMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_content_language(this: &R2HTTPMetadata, val: &str);` → `pub fn set_content_language(this: &R2HTTPMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_content_type(this: &R2HTTPMetadata, val: &str);` → `pub fn set_content_type(this: &R2HTTPMetadata, val: impl ::wasm_bindgen::JsStringLike);`

### `R2HTTPMetadataBuilder`

- `pub fn cache_control(self, val: &str) -> Self;` → `pub fn cache_control(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn content_disposition(self, val: &str) -> Self;` → `pub fn content_disposition(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn content_encoding(self, val: &str) -> Self;` → `pub fn content_encoding(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn content_language(self, val: &str) -> Self;` → `pub fn content_language(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn content_type(self, val: &str) -> Self;` → `pub fn content_type(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2ListOptions`

- added → `pub fn cursor_js_string(this: &R2ListOptions) -> Option<JsString>;`
- added → `pub fn delimiter_js_string(this: &R2ListOptions) -> Option<JsString>;`
- added → `pub fn include_js_string(this: &R2ListOptions) -> Option<Array<JsString>>;`
- added → `pub fn prefix_js_string(this: &R2ListOptions) -> Option<JsString>;`
- `pub fn set_cursor(this: &R2ListOptions, val: &str);` → `pub fn set_cursor(this: &R2ListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_delimiter(this: &R2ListOptions, val: &str);` → `pub fn set_delimiter(this: &R2ListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_include(this: &R2ListOptions, val: &[String]);` → `pub fn set_include(this: &R2ListOptions, val: &[impl ::wasm_bindgen::JsStringLike]);`
- `pub fn set_prefix(this: &R2ListOptions, val: &str);` → `pub fn set_prefix(this: &R2ListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_start_after(this: &R2ListOptions, val: &str);` → `pub fn set_start_after(this: &R2ListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn start_after_js_string(this: &R2ListOptions) -> Option<JsString>;`

### `R2ListOptionsBuilder`

- `pub fn cursor(self, val: &str) -> Self;` → `pub fn cursor(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn delimiter(self, val: &str) -> Self;` → `pub fn delimiter(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn include(self, val: &[String]) -> Self;` → `pub fn include(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`
- `pub fn prefix(self, val: &str) -> Self;` → `pub fn prefix(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn start_after(self, val: &str) -> Self;` → `pub fn start_after(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2MultipartOptions`

- `pub fn set_ssec_key_with_str(this: &R2MultipartOptions, val: &str);` → `pub fn set_ssec_key_with_str(this: &R2MultipartOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_storage_class(this: &R2MultipartOptions, val: &str);` → `pub fn set_storage_class(this: &R2MultipartOptions, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn storage_class_js_string(this: &R2MultipartOptions) -> Option<JsString>;`

### `R2MultipartOptionsBuilder`

- `pub fn ssec_key_with_str(self, val: &str) -> Self;` → `pub fn ssec_key_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn storage_class(self, val: &str) -> Self;` → `pub fn storage_class(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2MultipartUpload`

- added → `pub fn key_js_string(this: &R2MultipartUpload) -> JsString;`
- added → `pub fn upload_id_js_string(this: &R2MultipartUpload) -> JsString;`
- `pub async fn upload_part_with_str(this: &R2MultipartUpload, part_number: u16, value: &str) -> Result<R2UploadedPart, crate::Error>;` → `pub async fn upload_part_with_str(this: &R2MultipartUpload, part_number: u16, value: impl ::wasm_bindgen::JsStringLike) -> Result<R2UploadedPart, crate::Error>;`
- `pub async fn upload_part_with_str_and_options(this: &R2MultipartUpload, part_number: u16, value: &str, options: &R2UploadPartOptions) -> Result<R2UploadedPart, crate::Error>;` → `pub async fn upload_part_with_str_and_options(this: &R2MultipartUpload, part_number: u16, value: impl ::wasm_bindgen::JsStringLike, options: &R2UploadPartOptions) -> Result<R2UploadedPart, crate::Error>;`
- `pub fn upload_part_with_str_and_options_promise(this: &R2MultipartUpload, part_number: u16, value: &str, options: &R2UploadPartOptions) -> Result<Promise<R2UploadedPart>, crate::Error>;` → `pub fn upload_part_with_str_and_options_promise(this: &R2MultipartUpload, part_number: u16, value: impl ::wasm_bindgen::JsStringLike, options: &R2UploadPartOptions) -> Result<Promise<R2UploadedPart>, crate::Error>;`
- `pub fn upload_part_with_str_promise(this: &R2MultipartUpload, part_number: u16, value: &str) -> Result<Promise<R2UploadedPart>, crate::Error>;` → `pub fn upload_part_with_str_promise(this: &R2MultipartUpload, part_number: u16, value: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<R2UploadedPart>, crate::Error>;`

### `R2Object`

- added → `pub fn etag_js_string(this: &R2Object) -> JsString;`
- added → `pub fn http_etag_js_string(this: &R2Object) -> JsString;`
- added → `pub fn key_js_string(this: &R2Object) -> JsString;`
- added → `pub fn ssec_key_md5_js_string(this: &R2Object) -> Option<JsString>;`
- added → `pub fn storage_class_js_string(this: &R2Object) -> JsString;`
- added → `pub fn version_js_string(this: &R2Object) -> JsString;`

### `R2ObjectBody`

- `pub async fn json<T: ::wasm_bindgen::JsGeneric + ::wasm_bindgen::convert::FromWasmAbi>(this: &R2ObjectBody) -> Result<T, crate::Error>;` → `pub async fn json<T>(this: &R2ObjectBody) -> Result<T, crate::Error>;`
- `pub fn json_promise<T: ::wasm_bindgen::JsGeneric>(this: &R2ObjectBody) -> Result<Promise<T>, crate::Error>;` → `pub fn json_promise<T>(this: &R2ObjectBody) -> Result<Promise<T>, crate::Error>;`
- added → `pub async fn text_js_string(this: &R2ObjectBody) -> Result<JsString, crate::Error>;`

### `R2Objects`

- `pub fn builder_false(objects: &[R2Object], delimited_prefixes: &[String]) -> R2ObjectsBuilder;` → `pub fn builder_false(objects: &[R2Object], delimited_prefixes: &[impl ::wasm_bindgen::JsStringLike]) -> R2ObjectsBuilder;`
- `pub fn builder_true(objects: &[R2Object], delimited_prefixes: &[String]) -> R2ObjectsBuilder;` → `pub fn builder_true(objects: &[R2Object], delimited_prefixes: &[impl ::wasm_bindgen::JsStringLike]) -> R2ObjectsBuilder;`
- added → `pub fn cursor_js_string(this: &R2Objects) -> Option<JsString>;`
- added → `pub fn delimited_prefixes_js_string(this: &R2Objects) -> Array<JsString>;`
- `pub fn new_false(objects: &[R2Object], delimited_prefixes: &[String]) -> R2Objects;` → `pub fn new_false(objects: &[R2Object], delimited_prefixes: &[impl ::wasm_bindgen::JsStringLike]) -> R2Objects;`
- `pub fn new_true(objects: &[R2Object], delimited_prefixes: &[String]) -> R2Objects;` → `pub fn new_true(objects: &[R2Object], delimited_prefixes: &[impl ::wasm_bindgen::JsStringLike]) -> R2Objects;`
- `pub fn set_cursor(this: &R2Objects, val: &str);` → `pub fn set_cursor(this: &R2Objects, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_delimited_prefixes(this: &R2Objects, val: &[String]);` → `pub fn set_delimited_prefixes(this: &R2Objects, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `R2ObjectsBuilder`

- `pub fn cursor(self, val: &str) -> Self;` → `pub fn cursor(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2PutOptions`

- `pub fn set_md5_with_str(this: &R2PutOptions, val: &str);` → `pub fn set_md5_with_str(this: &R2PutOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_sha1_with_str(this: &R2PutOptions, val: &str);` → `pub fn set_sha1_with_str(this: &R2PutOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_sha256_with_str(this: &R2PutOptions, val: &str);` → `pub fn set_sha256_with_str(this: &R2PutOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_sha384_with_str(this: &R2PutOptions, val: &str);` → `pub fn set_sha384_with_str(this: &R2PutOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_sha512_with_str(this: &R2PutOptions, val: &str);` → `pub fn set_sha512_with_str(this: &R2PutOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_ssec_key_with_str(this: &R2PutOptions, val: &str);` → `pub fn set_ssec_key_with_str(this: &R2PutOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_storage_class(this: &R2PutOptions, val: &str);` → `pub fn set_storage_class(this: &R2PutOptions, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn storage_class_js_string(this: &R2PutOptions) -> Option<JsString>;`

### `R2PutOptionsBuilder`

- `pub fn md5_with_str(self, val: &str) -> Self;` → `pub fn md5_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn md5_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn md5_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn sha1_with_str(self, val: &str) -> Self;` → `pub fn sha1_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha1_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn sha1_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn sha256_with_str(self, val: &str) -> Self;` → `pub fn sha256_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha256_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn sha256_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn sha384_with_str(self, val: &str) -> Self;` → `pub fn sha384_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha384_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn sha384_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn sha512_with_str(self, val: &str) -> Self;` → `pub fn sha512_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha512_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn sha512_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn ssec_key_with_str(self, val: &str) -> Self;` → `pub fn ssec_key_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn storage_class(self, val: &str) -> Self;` → `pub fn storage_class(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2StringChecksums`

- added → `pub fn md5_js_string(this: &R2StringChecksums) -> Option<JsString>;`
- `pub fn set_md5(this: &R2StringChecksums, val: &str);` → `pub fn set_md5(this: &R2StringChecksums, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_sha1(this: &R2StringChecksums, val: &str);` → `pub fn set_sha1(this: &R2StringChecksums, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_sha256(this: &R2StringChecksums, val: &str);` → `pub fn set_sha256(this: &R2StringChecksums, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_sha384(this: &R2StringChecksums, val: &str);` → `pub fn set_sha384(this: &R2StringChecksums, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_sha512(this: &R2StringChecksums, val: &str);` → `pub fn set_sha512(this: &R2StringChecksums, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn sha1_js_string(this: &R2StringChecksums) -> Option<JsString>;`
- added → `pub fn sha256_js_string(this: &R2StringChecksums) -> Option<JsString>;`
- added → `pub fn sha384_js_string(this: &R2StringChecksums) -> Option<JsString>;`
- added → `pub fn sha512_js_string(this: &R2StringChecksums) -> Option<JsString>;`

### `R2StringChecksumsBuilder`

- `pub fn md5(self, val: &str) -> Self;` → `pub fn md5(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha1(self, val: &str) -> Self;` → `pub fn sha1(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha256(self, val: &str) -> Self;` → `pub fn sha256(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha384(self, val: &str) -> Self;` → `pub fn sha384(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn sha512(self, val: &str) -> Self;` → `pub fn sha512(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2UploadPartOptions`

- `pub fn set_ssec_key_with_str(this: &R2UploadPartOptions, val: &str);` → `pub fn set_ssec_key_with_str(this: &R2UploadPartOptions, val: impl ::wasm_bindgen::JsStringLike);`

### `R2UploadPartOptionsBuilder`

- `pub fn ssec_key_with_str(self, val: &str) -> Self;` → `pub fn ssec_key_with_str(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `R2UploadedPart`

- added → `pub fn etag_js_string(this: &R2UploadedPart) -> JsString;`
- `pub fn new(part_number: u16, etag: &str) -> R2UploadedPart;` → `pub fn new(part_number: u16, etag: impl ::wasm_bindgen::JsStringLike) -> R2UploadedPart;`
- `pub fn set_etag(this: &R2UploadedPart, val: &str);` → `pub fn set_etag(this: &R2UploadedPart, val: impl ::wasm_bindgen::JsStringLike);`

### `RateLimitOptions`

- added → `pub fn key_js_string(this: &RateLimitOptions) -> JsString;`
- `pub fn new(key: &str) -> RateLimitOptions;` → `pub fn new(key: impl ::wasm_bindgen::JsStringLike) -> RateLimitOptions;`
- `pub fn set_key(this: &RateLimitOptions, val: &str);` → `pub fn set_key(this: &RateLimitOptions, val: impl ::wasm_bindgen::JsStringLike);`

### `ReadableStream`

- `pub async fn cancel<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>) -> Result<(), crate::Error>;` → `pub async fn cancel<R>(this: &ReadableStream<R>) -> Result<(), crate::Error>;`
- `pub fn cancel_promise<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn cancel_promise<R>(this: &ReadableStream<R>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn cancel_with_reason<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, reason: &JsValue) -> Result<(), crate::Error>;` → `pub async fn cancel_with_reason<R>(this: &ReadableStream<R>, reason: &JsValue) -> Result<(), crate::Error>;`
- `pub fn cancel_with_reason_promise<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, reason: &JsValue) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn cancel_with_reason_promise<R>(this: &ReadableStream<R>, reason: &JsValue) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn get_reader<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>) -> Result<ReadableStreamDefaultReader<R>, JsException>;` → `pub fn get_reader<R>(this: &ReadableStream<R>) -> Result<ReadableStreamDefaultReader<R>, JsException>;`
- `pub fn get_reader_with_options<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, options: &ReadableStreamGetReaderOptions) -> Result<ReadableStreamBYOBReader, JsException>;` → `pub fn get_reader_with_options<R>(this: &ReadableStream<R>, options: &ReadableStreamGetReaderOptions) -> Result<ReadableStreamBYOBReader, JsException>;`
- `pub fn locked<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>) -> bool;` → `pub fn locked<R>(this: &ReadableStream<R>) -> bool;`
- `pub fn new() -> Result<ReadableStream, crate::Error>;` → `pub fn new<R>() -> Result<ReadableStream<R>, crate::Error>;`
- `pub fn new_with_underlying_byte_source(underlying_source: &UnderlyingByteSource) -> Result<ReadableStream, crate::Error>;` → `pub fn new_with_underlying_byte_source<R>(underlying_source: &UnderlyingByteSource) -> Result<ReadableStream<R>, crate::Error>;`
- `pub fn new_with_underlying_byte_source_and_queuing_strategy(underlying_source: &UnderlyingByteSource, strategy: &QueuingStrategy<Uint8Array>) -> Result<ReadableStream, crate::Error>;` → `pub fn new_with_underlying_byte_source_and_queuing_strategy<R>(underlying_source: &UnderlyingByteSource, strategy: &QueuingStrategy<Uint8Array>) -> Result<ReadableStream<R>, crate::Error>;`
- `pub fn new_with_underlying_source<R: ::wasm_bindgen::JsGeneric>(underlying_source: &UnderlyingSource<R>) -> Result<ReadableStream<R>, crate::Error>;` → `pub fn new_with_underlying_source<R>(underlying_source: &UnderlyingSource<R>) -> Result<ReadableStream<R>, crate::Error>;`
- `pub fn new_with_underlying_source_and_queuing_strategy<R: ::wasm_bindgen::JsGeneric>(underlying_source: &UnderlyingSource<R>, strategy: &QueuingStrategy<R>) -> Result<ReadableStream<R>, crate::Error>;` → `pub fn new_with_underlying_source_and_queuing_strategy<R>(underlying_source: &UnderlyingSource<R>, strategy: &QueuingStrategy<R>) -> Result<ReadableStream<R>, crate::Error>;`
- `pub fn pipe_through<R: ::wasm_bindgen::JsGeneric, T: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, transform: &ReadableWritablePair<T, R>) -> Result<ReadableStream<T>, crate::Error>;` → `pub fn pipe_through<R, T>(this: &ReadableStream<R>, transform: &ReadableWritablePair<T, R>) -> Result<ReadableStream<T>, crate::Error>;`
- `pub fn pipe_through_with_options<R: ::wasm_bindgen::JsGeneric, T: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, transform: &ReadableWritablePair<T, R>, options: &StreamPipeOptions) -> Result<ReadableStream<T>, crate::Error>;` → `pub fn pipe_through_with_options<R, T>(this: &ReadableStream<R>, transform: &ReadableWritablePair<T, R>, options: &StreamPipeOptions) -> Result<ReadableStream<T>, crate::Error>;`
- `pub async fn pipe_to<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, destination: &WritableStream<R>) -> Result<(), crate::Error>;` → `pub async fn pipe_to<R>(this: &ReadableStream<R>, destination: &WritableStream<R>) -> Result<(), crate::Error>;`
- `pub fn pipe_to_promise<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, destination: &WritableStream<R>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn pipe_to_promise<R>(this: &ReadableStream<R>, destination: &WritableStream<R>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn pipe_to_with_options<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, destination: &WritableStream<R>, options: &StreamPipeOptions) -> Result<(), crate::Error>;` → `pub async fn pipe_to_with_options<R>(this: &ReadableStream<R>, destination: &WritableStream<R>, options: &StreamPipeOptions) -> Result<(), crate::Error>;`
- `pub fn pipe_to_with_options_promise<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, destination: &WritableStream<R>, options: &StreamPipeOptions) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn pipe_to_with_options_promise<R>(this: &ReadableStream<R>, destination: &WritableStream<R>, options: &StreamPipeOptions) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn tee<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>) -> Result<ArrayTuple<(ReadableStream<R>, ReadableStream<R>)>, crate::Error>;` → `pub fn tee<R>(this: &ReadableStream<R>) -> Result<ArrayTuple<(ReadableStream<R>, ReadableStream<R>)>, crate::Error>;`
- `pub fn values<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>) -> Result<AsyncIterator<R>, crate::Error>;` → `pub fn values<R>(this: &ReadableStream<R>) -> Result<AsyncIterator<R>, crate::Error>;`
- `pub fn values_with_options<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStream<R>, options: &ReadableStreamValuesOptions) -> Result<AsyncIterator<R>, crate::Error>;` → `pub fn values_with_options<R>(this: &ReadableStream<R>, options: &ReadableStreamValuesOptions) -> Result<AsyncIterator<R>, crate::Error>;`
- `pub type ReadableStream<R: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type ReadableStream<R = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `ReadableStreamBYOBReader`

- `pub async fn read<T: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamBYOBReader, view: &T) -> Result<ReadableStreamReadResult<T>, crate::Error>;` → `pub async fn read<T>(this: &ReadableStreamBYOBReader, view: T) -> Result<ReadableStreamReadResult<T>, crate::Error>;`
- `pub async fn read_at_least<T: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamBYOBReader, min_elements: f64, view: &T) -> Result<ReadableStreamReadResult<T>, crate::Error>;` → `pub async fn read_at_least<T>(this: &ReadableStreamBYOBReader, min_elements: f64, view: T) -> Result<ReadableStreamReadResult<T>, crate::Error>;`
- `pub fn read_at_least_promise<T: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamBYOBReader, min_elements: f64, view: &T) -> Result<Promise<ReadableStreamReadResult<T>>, crate::Error>;` → `pub fn read_at_least_promise<T>(this: &ReadableStreamBYOBReader, min_elements: f64, view: T) -> Result<Promise<ReadableStreamReadResult<T>>, crate::Error>;`
- `pub fn read_promise<T: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamBYOBReader, view: &T) -> Result<Promise<ReadableStreamReadResult<T>>, crate::Error>;` → `pub fn read_promise<T>(this: &ReadableStreamBYOBReader, view: T) -> Result<Promise<ReadableStreamReadResult<T>>, crate::Error>;`

### `ReadableStreamDefaultController`

- `pub fn close<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultController<R>) -> Result<(), crate::Error>;` → `pub fn close<R>(this: &ReadableStreamDefaultController<R>) -> Result<(), crate::Error>;`
- `pub fn desired_size<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultController<R>) -> Option<f64>;` → `pub fn desired_size<R>(this: &ReadableStreamDefaultController<R>) -> Option<f64>;`
- `pub fn enqueue<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultController<R>) -> Result<(), crate::Error>;` → `pub fn enqueue<R>(this: &ReadableStreamDefaultController<R>) -> Result<(), crate::Error>;`
- `pub fn enqueue_with_chunk<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultController<R>, chunk: &R) -> Result<(), crate::Error>;` → `pub fn enqueue_with_chunk<R>(this: &ReadableStreamDefaultController<R>, chunk: R) -> Result<(), crate::Error>;`
- `pub fn error<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultController<R>, reason: &JsValue) -> Result<(), crate::Error>;` → `pub fn error<R>(this: &ReadableStreamDefaultController<R>, reason: &JsValue) -> Result<(), crate::Error>;`
- `pub type ReadableStreamDefaultController<R: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type ReadableStreamDefaultController<R = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `ReadableStreamDefaultReader`

- `pub async fn cancel<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultReader<R>) -> Result<(), crate::Error>;` → `pub async fn cancel<R>(this: &ReadableStreamDefaultReader<R>) -> Result<(), crate::Error>;`
- `pub fn cancel_promise<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultReader<R>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn cancel_promise<R>(this: &ReadableStreamDefaultReader<R>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn cancel_with_reason<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultReader<R>, reason: &JsValue) -> Result<(), crate::Error>;` → `pub async fn cancel_with_reason<R>(this: &ReadableStreamDefaultReader<R>, reason: &JsValue) -> Result<(), crate::Error>;`
- `pub fn cancel_with_reason_promise<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultReader<R>, reason: &JsValue) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn cancel_with_reason_promise<R>(this: &ReadableStreamDefaultReader<R>, reason: &JsValue) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn closed<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultReader<R>) -> Promise<Undefined>;` → `pub fn closed<R>(this: &ReadableStreamDefaultReader<R>) -> Promise<Undefined>;`
- `pub fn new(stream: &ReadableStream) -> Result<ReadableStreamDefaultReader, crate::Error>;` → `pub fn new<R>(stream: &ReadableStream) -> Result<ReadableStreamDefaultReader<R>, crate::Error>;`
- `pub async fn read<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultReader<R>) -> Result<ReadableStreamReadResult<R>, crate::Error>;` → `pub async fn read<R>(this: &ReadableStreamDefaultReader<R>) -> Result<ReadableStreamReadResult<R>, crate::Error>;`
- `pub fn read_promise<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultReader<R>) -> Result<Promise<ReadableStreamReadResult<R>>, crate::Error>;` → `pub fn read_promise<R>(this: &ReadableStreamDefaultReader<R>) -> Result<Promise<ReadableStreamReadResult<R>>, crate::Error>;`
- `pub fn release_lock<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamDefaultReader<R>) -> Result<(), crate::Error>;` → `pub fn release_lock<R>(this: &ReadableStreamDefaultReader<R>) -> Result<(), crate::Error>;`
- `pub type ReadableStreamDefaultReader<R: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type ReadableStreamDefaultReader<R = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `ReadableStreamGetReaderOptions`

- added → `pub fn mode_js_string(this: &ReadableStreamGetReaderOptions) -> JsString;`
- `pub fn set_mode(this: &ReadableStreamGetReaderOptions, val: &str);` → `pub fn set_mode(this: &ReadableStreamGetReaderOptions, val: impl ::wasm_bindgen::JsStringLike);`

### `ReadableStreamReadResult`

- `pub fn done<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamReadResult<R>) -> bool;` → `pub fn done<R>(this: &ReadableStreamReadResult<R>) -> bool;`
- `pub fn set_done<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamReadResult<R>, val: bool);` → `pub fn set_done<R>(this: &ReadableStreamReadResult<R>, val: bool);`
- `pub fn set_value<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamReadResult<R>, val: Option<R>);` → `pub fn set_value<R>(this: &ReadableStreamReadResult<R>, val: Option<R>);`
- `pub fn value<R: ::wasm_bindgen::JsGeneric>(this: &ReadableStreamReadResult<R>) -> Option<R>;` → `pub fn value<R>(this: &ReadableStreamReadResult<R>) -> Option<R>;`
- `impl<R: ::wasm_bindgen::JsGeneric> ReadableStreamReadResult<R>` → `impl<R: ::wasm_bindgen::convert::IntoWasmAbi> ReadableStreamReadResult<R>`
- `pub type ReadableStreamReadResult<R: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type ReadableStreamReadResult<R = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `ReadableStreamReadResultBuilder`

- `impl<R: ::wasm_bindgen::JsGeneric> From<ReadableStreamReadResult<R>> for ReadableStreamReadResultBuilder<R>` → `impl<R: ::wasm_bindgen::convert::IntoWasmAbi> From<ReadableStreamReadResult<R>> for ReadableStreamReadResultBuilder<R>`
- `impl<R: ::wasm_bindgen::JsGeneric> ReadableStreamReadResultBuilder<R>` → `impl<R: ::wasm_bindgen::convert::IntoWasmAbi> ReadableStreamReadResultBuilder<R>`
- `pub struct ReadableStreamReadResultBuilder<R: ::wasm_bindgen::JsGeneric> { inner: ReadableStreamReadResult<R> }` → `pub struct ReadableStreamReadResultBuilder<R> { inner: ReadableStreamReadResult<R> }`

### `ReadableWritablePair`

- `pub fn readable<R: ::wasm_bindgen::JsGeneric, W: ::wasm_bindgen::JsGeneric>(this: &ReadableWritablePair<R, W>) -> ReadableStream<R>;` → `pub fn readable<R, W>(this: &ReadableWritablePair<R, W>) -> ReadableStream<R>;`
- `pub fn set_readable<R: ::wasm_bindgen::JsGeneric, W: ::wasm_bindgen::JsGeneric>(this: &ReadableWritablePair<R, W>, val: &ReadableStream<R>);` → `pub fn set_readable<R, W>(this: &ReadableWritablePair<R, W>, val: &ReadableStream<R>);`
- `pub fn set_writable<R: ::wasm_bindgen::JsGeneric, W: ::wasm_bindgen::JsGeneric>(this: &ReadableWritablePair<R, W>, val: &WritableStream<W>);` → `pub fn set_writable<R, W>(this: &ReadableWritablePair<R, W>, val: &WritableStream<W>);`
- `pub fn writable<R: ::wasm_bindgen::JsGeneric, W: ::wasm_bindgen::JsGeneric>(this: &ReadableWritablePair<R, W>) -> WritableStream<W>;` → `pub fn writable<R, W>(this: &ReadableWritablePair<R, W>) -> WritableStream<W>;`
- `impl<R: ::wasm_bindgen::JsGeneric, W: ::wasm_bindgen::JsGeneric> ReadableWritablePair<R, W>` → `impl<R: ::wasm_bindgen::convert::IntoWasmAbi, W: ::wasm_bindgen::convert::IntoWasmAbi> ReadableWritablePair<R, W>`
- `pub type ReadableWritablePair<R: ::wasm_bindgen::JsGeneric = JsValue, W: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type ReadableWritablePair<R = JsValue, W = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `Request`

- `pub fn cache<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> Option<CacheKind>;` → `pub fn cache<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> Option<CacheKind>;`
- `pub fn cf<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> Option<Cf>;` → `pub fn cf<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> Option<Cf>;`
- `pub fn clone<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> Result<Request<CfHostMetadata, Cf>, JsException>;` → `pub fn clone<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> Result<Request<CfHostMetadata, Cf>, JsException>;`
- `pub fn fetcher<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> Option<Fetcher>;` → `pub fn fetcher<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> Option<Fetcher>;`
- `pub fn headers<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> Headers;` → `pub fn headers<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> Headers;`
- `pub fn integrity<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> String;` → `pub fn integrity<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> String;`
- added → `pub fn integrity_js_string<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> JsString;`
- `pub fn keepalive<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> bool;` → `pub fn keepalive<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> bool;`
- `pub fn method<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> String;` → `pub fn method<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> String;`
- added → `pub fn method_js_string<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> JsString;`
- `pub fn new(input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>) -> Result<Request, JsException>;` → `pub fn new<CfHostMetadata, Cf>(input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>) -> Result<Request<CfHostMetadata, Cf>, JsException>;`
- `pub fn new_with_request_and_init<Cf: ::wasm_bindgen::JsGeneric>(input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>, init: &RequestInit<Cf>) -> Result<Request<JsValue, Cf>, JsException>;` → `pub fn new_with_request_and_init<CfHostMetadata, Cf>(input: &Request<JsValue, IncomingRequestCfProperties<JsValue>>, init: &RequestInit<Cf>) -> Result<Request<CfHostMetadata, Cf>, JsException>;`
- `pub fn new_with_str(input: &str) -> Result<Request, JsException>;` → `pub fn new_with_str<CfHostMetadata, Cf>(input: impl ::wasm_bindgen::JsStringLike) -> Result<Request<CfHostMetadata, Cf>, JsException>;`
- `pub fn new_with_str_and_init<Cf: ::wasm_bindgen::JsGeneric>(input: &str, init: &RequestInit<Cf>) -> Result<Request<JsValue, Cf>, JsException>;` → `pub fn new_with_str_and_init<CfHostMetadata, Cf>(input: impl ::wasm_bindgen::JsStringLike, init: &RequestInit<Cf>) -> Result<Request<CfHostMetadata, Cf>, JsException>;`
- `pub fn new_with_url(input: &URL) -> Result<Request, JsException>;` → `pub fn new_with_url<CfHostMetadata, Cf>(input: &URL) -> Result<Request<CfHostMetadata, Cf>, JsException>;`
- `pub fn new_with_url_and_init<Cf: ::wasm_bindgen::JsGeneric>(input: &URL, init: &RequestInit<Cf>) -> Result<Request<JsValue, Cf>, JsException>;` → `pub fn new_with_url_and_init<CfHostMetadata, Cf>(input: &URL, init: &RequestInit<Cf>) -> Result<Request<CfHostMetadata, Cf>, JsException>;`
- `pub fn redirect<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> String;` → `pub fn redirect<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> String;`
- added → `pub fn redirect_js_string<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> JsString;`
- `pub fn set_cache<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: CacheKind);` → `pub fn set_cache<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: CacheKind);`
- `pub fn set_cf<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: &Cf);` → `pub fn set_cf<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: Cf);`
- `pub fn set_fetcher<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: &JsNullable<Fetcher>);` → `pub fn set_fetcher<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: &JsNullable<Fetcher>);`
- `pub fn set_headers<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: &Headers);` → `pub fn set_headers<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: &Headers);`
- `pub fn set_integrity<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: &str);` → `pub fn set_integrity<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_keepalive<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: bool);` → `pub fn set_keepalive<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: bool);`
- `pub fn set_method<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: &str);` → `pub fn set_method<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_redirect<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: &str);` → `pub fn set_redirect<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_signal<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: &AbortSignal);` → `pub fn set_signal<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: &AbortSignal);`
- `pub fn set_url<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>, val: &str);` → `pub fn set_url<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn signal<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> AbortSignal;` → `pub fn signal<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> AbortSignal;`
- `pub fn url<CfHostMetadata: ::wasm_bindgen::JsGeneric, Cf: ::wasm_bindgen::JsGeneric>(this: &Request<CfHostMetadata, Cf>) -> String;` → `pub fn url<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> String;`
- added → `pub fn url_js_string<CfHostMetadata, Cf>(this: &Request<CfHostMetadata, Cf>) -> JsString;`
- `pub type Request<CfHostMetadata: ::wasm_bindgen::JsGeneric = JsValue, Cf: ::wasm_bindgen::JsGeneric = IncomingRequestCfProperties<CfHostMetadata>>; extends(Body, Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type Request<CfHostMetadata = JsValue, Cf = IncomingRequestCfProperties<CfHostMetadata>>; extends(Body, Object) derive(Debug, Clone, PartialEq, Eq)`

### `RequestInit`

- `pub fn body<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> JsValue;` → `pub fn body<Cf>(this: &RequestInit<Cf>) -> JsValue;`
- `pub fn cache<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> Option<CacheKind>;` → `pub fn cache<Cf>(this: &RequestInit<Cf>) -> Option<CacheKind>;`
- `pub fn cf<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> Option<Cf>;` → `pub fn cf<Cf>(this: &RequestInit<Cf>) -> Option<Cf>;`
- `pub fn encode_response_body<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> Option<EncodeBodyKind>;` → `pub fn encode_response_body<Cf>(this: &RequestInit<Cf>) -> Option<EncodeBodyKind>;`
- `pub fn fetcher<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> Option<Fetcher>;` → `pub fn fetcher<Cf>(this: &RequestInit<Cf>) -> Option<Fetcher>;`
- `pub fn headers<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> JsValue;` → `pub fn headers<Cf>(this: &RequestInit<Cf>) -> JsValue;`
- `pub fn integrity<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> Option<String>;` → `pub fn integrity<Cf>(this: &RequestInit<Cf>) -> Option<String>;`
- added → `pub fn integrity_js_string<Cf>(this: &RequestInit<Cf>) -> Option<JsString>;`
- `pub fn method<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> Option<String>;` → `pub fn method<Cf>(this: &RequestInit<Cf>) -> Option<String>;`
- added → `pub fn method_js_string<Cf>(this: &RequestInit<Cf>) -> Option<JsString>;`
- `pub fn redirect<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> Option<String>;` → `pub fn redirect<Cf>(this: &RequestInit<Cf>) -> Option<String>;`
- added → `pub fn redirect_js_string<Cf>(this: &RequestInit<Cf>) -> Option<JsString>;`
- `pub fn set_body<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsNullable<ReadableStream<Uint8Array>>);` → `pub fn set_body<Cf>(this: &RequestInit<Cf>, val: &JsNullable<ReadableStream<Uint8Array>>);`
- `pub fn set_body_with_array_buffer<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsNullable<ArrayBuffer>);` → `pub fn set_body_with_array_buffer<Cf>(this: &RequestInit<Cf>, val: &JsNullable<ArrayBuffer>);`
- `pub fn set_body_with_blob<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsNullable<Blob>);` → `pub fn set_body_with_blob<Cf>(this: &RequestInit<Cf>, val: &JsNullable<Blob>);`
- `pub fn set_body_with_form_data<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsNullable<FormData>);` → `pub fn set_body_with_form_data<Cf>(this: &RequestInit<Cf>, val: &JsNullable<FormData>);`
- `pub fn set_body_with_iterable<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsValue);` → `pub fn set_body_with_iterable<Cf>(this: &RequestInit<Cf>, val: &JsValue);`
- `pub fn set_body_with_str<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsNullable<JsString>);` → `pub fn set_body_with_str<Cf>(this: &RequestInit<Cf>, val: &JsNullable<JsString>);`
- `pub fn set_body_with_typed_array<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsNullable<Object>);` → `pub fn set_body_with_typed_array<Cf>(this: &RequestInit<Cf>, val: &JsNullable<Object>);`
- `pub fn set_body_with_url_search_params<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsNullable<URLSearchParams>);` → `pub fn set_body_with_url_search_params<Cf>(this: &RequestInit<Cf>, val: &JsNullable<URLSearchParams>);`
- `pub fn set_cache<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: CacheKind);` → `pub fn set_cache<Cf>(this: &RequestInit<Cf>, val: CacheKind);`
- `pub fn set_cf<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &Cf);` → `pub fn set_cf<Cf>(this: &RequestInit<Cf>, val: Cf);`
- `pub fn set_encode_response_body<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: EncodeBodyKind);` → `pub fn set_encode_response_body<Cf>(this: &RequestInit<Cf>, val: EncodeBodyKind);`
- `pub fn set_fetcher<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsNullable<Fetcher>);` → `pub fn set_fetcher<Cf>(this: &RequestInit<Cf>, val: &JsNullable<Fetcher>);`
- `pub fn set_headers<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &Headers);` → `pub fn set_headers<Cf>(this: &RequestInit<Cf>, val: &Headers);`
- `pub fn set_headers_with_iterable<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsValue);` → `pub fn set_headers_with_iterable<Cf>(this: &RequestInit<Cf>, val: &JsValue);`
- `pub fn set_headers_with_record<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &Object<JsString>);` → `pub fn set_headers_with_record<Cf>(this: &RequestInit<Cf>, val: &Object<JsString>);`
- `pub fn set_integrity<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &str);` → `pub fn set_integrity<Cf>(this: &RequestInit<Cf>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_method<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &str);` → `pub fn set_method<Cf>(this: &RequestInit<Cf>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_redirect<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &str);` → `pub fn set_redirect<Cf>(this: &RequestInit<Cf>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_signal<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>, val: &JsNullable<AbortSignal>);` → `pub fn set_signal<Cf>(this: &RequestInit<Cf>, val: &JsNullable<AbortSignal>);`
- `pub fn signal<Cf: ::wasm_bindgen::JsGeneric>(this: &RequestInit<Cf>) -> Option<AbortSignal>;` → `pub fn signal<Cf>(this: &RequestInit<Cf>) -> Option<AbortSignal>;`
- `impl<Cf: ::wasm_bindgen::JsGeneric> RequestInit<Cf>` → `impl<Cf: ::wasm_bindgen::convert::IntoWasmAbi> RequestInit<Cf>`
- `pub type RequestInit<Cf: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type RequestInit<Cf = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `RequestInitBuilder`

- `pub fn cf(self, val: &Cf) -> Self;` → `pub fn cf(self, val: Cf) -> Self;`
- `pub fn integrity(self, val: &str) -> Self;` → `pub fn integrity(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn method(self, val: &str) -> Self;` → `pub fn method(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn redirect(self, val: &str) -> Self;` → `pub fn redirect(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `impl<Cf: ::wasm_bindgen::JsGeneric> From<RequestInit<Cf>> for RequestInitBuilder<Cf>` → `impl<Cf: ::wasm_bindgen::convert::IntoWasmAbi> From<RequestInit<Cf>> for RequestInitBuilder<Cf>`
- `impl<Cf: ::wasm_bindgen::JsGeneric> RequestInitBuilder<Cf>` → `impl<Cf: ::wasm_bindgen::convert::IntoWasmAbi> RequestInitBuilder<Cf>`
- `pub struct RequestInitBuilder<Cf: ::wasm_bindgen::JsGeneric> { inner: RequestInit<Cf> }` → `pub struct RequestInitBuilder<Cf> { inner: RequestInit<Cf> }`

### `RequestInitCfProperties`

- added → `pub fn cache_control_js_string(this: &RequestInitCfProperties) -> Option<JsString>;`
- added → `pub fn cache_key_js_string(this: &RequestInitCfProperties) -> Option<JsString>;`
- added → `pub fn cache_tags_js_string(this: &RequestInitCfProperties) -> Option<Array<JsString>>;`
- added → `pub fn resolve_override_js_string(this: &RequestInitCfProperties) -> Option<JsString>;`
- `pub fn set_cache_control(this: &RequestInitCfProperties, val: &str);` → `pub fn set_cache_control(this: &RequestInitCfProperties, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cache_key(this: &RequestInitCfProperties, val: &str);` → `pub fn set_cache_key(this: &RequestInitCfProperties, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_cache_tags(this: &RequestInitCfProperties, val: &[String]);` → `pub fn set_cache_tags(this: &RequestInitCfProperties, val: &[impl ::wasm_bindgen::JsStringLike]);`
- `pub fn set_resolve_override(this: &RequestInitCfProperties, val: &str);` → `pub fn set_resolve_override(this: &RequestInitCfProperties, val: impl ::wasm_bindgen::JsStringLike);`

### `RequestInitCfPropertiesBuilder`

- `pub fn cache_control(self, val: &str) -> Self;` → `pub fn cache_control(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn cache_key(self, val: &str) -> Self;` → `pub fn cache_key(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn cache_tags(self, val: &[String]) -> Self;` → `pub fn cache_tags(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`
- `pub fn resolve_override(self, val: &str) -> Self;` → `pub fn resolve_override(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `RequestInitCfPropertiesImage`

- added → `pub fn compression_js_string(this: &RequestInitCfPropertiesImage) -> Option<JsString>;`
- added → `pub fn origin_auth_js_string(this: &RequestInitCfPropertiesImage) -> Option<JsString>;`
- `pub fn set_compression(this: &RequestInitCfPropertiesImage, val: &str);` → `pub fn set_compression(this: &RequestInitCfPropertiesImage, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_origin_auth(this: &RequestInitCfPropertiesImage, val: &str);` → `pub fn set_origin_auth(this: &RequestInitCfPropertiesImage, val: impl ::wasm_bindgen::JsStringLike);`

### `RequestInitCfPropertiesImageBuilder`

- `pub fn background(self, val: &str) -> Self;` → `pub fn background(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn compression(self, val: &str) -> Self;` → `pub fn compression(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn origin_auth(self, val: &str) -> Self;` → `pub fn origin_auth(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn segment(self, val: &str) -> Self;` → `pub fn segment(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `RequestInitCfPropertiesImageDraw`

- `pub fn builder(url: &str) -> RequestInitCfPropertiesImageDrawBuilder;` → `pub fn builder(url: impl ::wasm_bindgen::JsStringLike) -> RequestInitCfPropertiesImageDrawBuilder;`
- `pub fn new(url: &str) -> RequestInitCfPropertiesImageDraw;` → `pub fn new(url: impl ::wasm_bindgen::JsStringLike) -> RequestInitCfPropertiesImageDraw;`
- `pub fn set_url(this: &RequestInitCfPropertiesImageDraw, val: &str);` → `pub fn set_url(this: &RequestInitCfPropertiesImageDraw, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn url_js_string(this: &RequestInitCfPropertiesImageDraw) -> JsString;`

### `RequestInitCfPropertiesImageDrawBuilder`

- `pub fn background(self, val: &str) -> Self;` → `pub fn background(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn segment(self, val: &str) -> Self;` → `pub fn segment(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `RequestInitCfPropertiesVaryAcceptHeader`

- added → `pub fn media_types_js_string(this: &RequestInitCfPropertiesVaryAcceptHeader) -> Option<Array<JsString>>;`
- `pub fn set_media_types(this: &RequestInitCfPropertiesVaryAcceptHeader, val: &[String]);` → `pub fn set_media_types(this: &RequestInitCfPropertiesVaryAcceptHeader, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `RequestInitCfPropertiesVaryAcceptHeaderBuilder`

- `pub fn media_types(self, val: &[String]) -> Self;` → `pub fn media_types(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `RequestInitCfPropertiesVaryAcceptLanguageHeader`

- added → `pub fn languages_js_string(this: &RequestInitCfPropertiesVaryAcceptLanguageHeader) -> Option<Array<JsString>>;`
- `pub fn set_languages(this: &RequestInitCfPropertiesVaryAcceptLanguageHeader, val: &[String]);` → `pub fn set_languages(this: &RequestInitCfPropertiesVaryAcceptLanguageHeader, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `RequestInitCfPropertiesVaryAcceptLanguageHeaderBuilder`

- `pub fn languages(self, val: &[String]) -> Self;` → `pub fn languages(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `Response`

- `pub fn redirect(url: &str) -> Result<Response, JsException>;` → `pub fn redirect(url: impl ::wasm_bindgen::JsStringLike) -> Result<Response, JsException>;`
- `pub fn redirect_with_status(url: &str, status: u16) -> Result<Response, JsException>;` → `pub fn redirect_with_status(url: impl ::wasm_bindgen::JsStringLike, status: u16) -> Result<Response, JsException>;`
- `pub fn set_status_text(this: &Response, val: &str);` → `pub fn set_status_text(this: &Response, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_url(this: &Response, val: &str);` → `pub fn set_url(this: &Response, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn status_text_js_string(this: &Response) -> JsString;`
- added → `pub fn url_js_string(this: &Response) -> JsString;`

### `ResponseInit`

- `pub fn set_status_text(this: &ResponseInit, val: &str);` → `pub fn set_status_text(this: &ResponseInit, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn status_text_js_string(this: &ResponseInit) -> Option<JsString>;`

### `ResponseInitBuilder`

- `pub fn status_text(self, val: &str) -> Self;` → `pub fn status_text(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `RsaOtherPrimesInfo`

- added → `pub fn d_js_string(this: &RsaOtherPrimesInfo) -> Option<JsString>;`
- added → `pub fn r_js_string(this: &RsaOtherPrimesInfo) -> Option<JsString>;`
- `pub fn set_d(this: &RsaOtherPrimesInfo, val: &str);` → `pub fn set_d(this: &RsaOtherPrimesInfo, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_r(this: &RsaOtherPrimesInfo, val: &str);` → `pub fn set_r(this: &RsaOtherPrimesInfo, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_t(this: &RsaOtherPrimesInfo, val: &str);` → `pub fn set_t(this: &RsaOtherPrimesInfo, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn t_js_string(this: &RsaOtherPrimesInfo) -> Option<JsString>;`

### `RsaOtherPrimesInfoBuilder`

- `pub fn d(self, val: &str) -> Self;` → `pub fn d(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn r(self, val: &str) -> Self;` → `pub fn r(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn t(self, val: &str) -> Self;` → `pub fn t(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `ScheduledController`

- added → `pub fn cron_js_string(this: &ScheduledController) -> JsString;`

### `ScriptVersion`

- added → `pub fn id_js_string(this: &ScriptVersion) -> Option<JsString>;`
- added → `pub fn message_js_string(this: &ScriptVersion) -> Option<JsString>;`
- `pub fn set_id(this: &ScriptVersion, val: &str);` → `pub fn set_id(this: &ScriptVersion, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_message(this: &ScriptVersion, val: &str);` → `pub fn set_message(this: &ScriptVersion, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_tag(this: &ScriptVersion, val: &str);` → `pub fn set_tag(this: &ScriptVersion, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn tag_js_string(this: &ScriptVersion) -> Option<JsString>;`

### `ScriptVersionBuilder`

- `pub fn id(self, val: &str) -> Self;` → `pub fn id(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn message(self, val: &str) -> Self;` → `pub fn message(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn tag(self, val: &str) -> Self;` → `pub fn tag(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `SecretsStoreSecret`

- added → `pub async fn get_js_string(this: &SecretsStoreSecret) -> Result<JsString, crate::Error>;`

### `SocketAddress`

- added → `pub fn hostname_js_string(this: &SocketAddress) -> JsString;`
- `pub fn new(hostname: &str, port: u16) -> SocketAddress;` → `pub fn new(hostname: impl ::wasm_bindgen::JsStringLike, port: u16) -> SocketAddress;`
- `pub fn set_hostname(this: &SocketAddress, val: &str);` → `pub fn set_hostname(this: &SocketAddress, val: impl ::wasm_bindgen::JsStringLike);`

### `SocketInfo`

- added → `pub fn local_address_js_string(this: &SocketInfo) -> Option<JsString>;`
- added → `pub fn remote_address_js_string(this: &SocketInfo) -> Option<JsString>;`
- `pub fn set_local_address(this: &SocketInfo, val: &str);` → `pub fn set_local_address(this: &SocketInfo, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_remote_address(this: &SocketInfo, val: &str);` → `pub fn set_remote_address(this: &SocketInfo, val: impl ::wasm_bindgen::JsStringLike);`

### `SocketInfoBuilder`

- `pub fn local_address(self, val: &str) -> Self;` → `pub fn local_address(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn remote_address(self, val: &str) -> Self;` → `pub fn remote_address(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `SocketOptions`

- added → `pub fn secure_transport_js_string(this: &SocketOptions) -> Option<JsString>;`
- `pub fn set_high_water_mark(this: &SocketOptions, val: f64);` → `pub fn set_high_water_mark(this: &SocketOptions, val: impl ::js_sys::JsBigIntOrNumberLike);`
- `pub fn set_high_water_mark_with_big_int(this: &SocketOptions, val: &BigInt);` → removed
- `pub fn set_secure_transport(this: &SocketOptions, val: &str);` → `pub fn set_secure_transport(this: &SocketOptions, val: impl ::wasm_bindgen::JsStringLike);`

### `SocketOptionsBuilder`

- `pub fn high_water_mark(self, val: f64) -> Self;` → `pub fn high_water_mark(self, val: impl ::js_sys::JsBigIntOrNumberLike) -> Self;`
- `pub fn high_water_mark_with_big_int(self, val: &BigInt) -> Self;` → removed
- `pub fn secure_transport(self, val: &str) -> Self;` → `pub fn secure_transport(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `Span`

- `pub fn set_attribute(this: &Span, key: &str, value: bool) -> Result<JsValue, crate::Error>;` → `pub fn set_attribute(this: &Span, key: impl ::wasm_bindgen::JsStringLike, value: impl ::js_sys::JsBooleanOrNumberOrStringLike) -> Result<JsValue, crate::Error>;`
- `pub fn set_attribute_with_f64(this: &Span, key: &str, value: f64) -> Result<JsValue, crate::Error>;` → removed
- `pub fn set_attribute_with_str(this: &Span, key: &str, value: &str) -> Result<JsValue, crate::Error>;` → removed

### `SqlStorage`

- `pub fn exec(this: &SqlStorage, query: &str, bindings: &[JsValue]) -> Result<SqlStorageCursor<JsValue>, crate::Error>; [variadic]` → `pub fn exec(this: &SqlStorage, query: impl ::wasm_bindgen::JsStringLike, bindings: &[JsValue]) -> Result<SqlStorageCursor<JsValue>, crate::Error>; [variadic]`

### `SqlStorageCursor`

- `pub fn column_names<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursor<T>) -> Vec<String>;` → `pub fn column_names<T>(this: &SqlStorageCursor<T>) -> Vec<String>;`
- added → `pub fn column_names_js_string<T>(this: &SqlStorageCursor<T>) -> Array<JsString>;`
- `pub fn next<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursor<T>) -> Result<SqlStorageCursorNextReturn<T>, crate::Error>;` → `pub fn next<T>(this: &SqlStorageCursor<T>) -> Result<SqlStorageCursorNextReturn<T>, crate::Error>;`
- `pub fn one<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursor<T>) -> Result<T, crate::Error>;` → `pub fn one<T>(this: &SqlStorageCursor<T>) -> Result<T, crate::Error>;`
- `pub fn raw<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursor<T>) -> Iterator;` → `pub fn raw<T>(this: &SqlStorageCursor<T>) -> Iterator;`
- `pub fn rows_read<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursor<T>) -> usize;` → `pub fn rows_read<T>(this: &SqlStorageCursor<T>) -> usize;`
- `pub fn rows_written<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursor<T>) -> usize;` → `pub fn rows_written<T>(this: &SqlStorageCursor<T>) -> usize;`
- `pub fn set_column_names<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursor<T>, val: &[String]);` → `pub fn set_column_names<T>(this: &SqlStorageCursor<T>, val: &[impl ::wasm_bindgen::JsStringLike]);`
- `pub fn to_array<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursor<T>) -> Result<Vec<T>, crate::Error>;` → `pub fn to_array<T>(this: &SqlStorageCursor<T>) -> Result<Vec<T>, crate::Error>;`
- `pub type SqlStorageCursor<T: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type SqlStorageCursor<T = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `SqlStorageCursorNextReturn`

- `pub fn done<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursorNextReturn<T>) -> Option<bool>;` → `pub fn done<T>(this: &SqlStorageCursorNextReturn<T>) -> Option<bool>;`
- `pub fn set_done<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursorNextReturn<T>, val: bool);` → `pub fn set_done<T>(this: &SqlStorageCursorNextReturn<T>, val: bool);`
- `pub fn set_value<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursorNextReturn<T>, val: Option<T>);` → `pub fn set_value<T>(this: &SqlStorageCursorNextReturn<T>, val: Option<T>);`
- `pub fn value<T: ::wasm_bindgen::JsGeneric>(this: &SqlStorageCursorNextReturn<T>) -> Option<T>;` → `pub fn value<T>(this: &SqlStorageCursorNextReturn<T>) -> Option<T>;`
- `impl<T: ::wasm_bindgen::JsGeneric> SqlStorageCursorNextReturn<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> SqlStorageCursorNextReturn<T>`
- `pub type SqlStorageCursorNextReturn<T: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type SqlStorageCursorNextReturn<T = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `SqlStorageCursorNextReturnBuilder`

- `impl<T: ::wasm_bindgen::JsGeneric> From<SqlStorageCursorNextReturn<T>> for SqlStorageCursorNextReturnBuilder<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> From<SqlStorageCursorNextReturn<T>> for SqlStorageCursorNextReturnBuilder<T>`
- `impl<T: ::wasm_bindgen::JsGeneric> SqlStorageCursorNextReturnBuilder<T>` → `impl<T: ::wasm_bindgen::convert::IntoWasmAbi> SqlStorageCursorNextReturnBuilder<T>`
- `pub struct SqlStorageCursorNextReturnBuilder<T: ::wasm_bindgen::JsGeneric> { inner: SqlStorageCursorNextReturn<T> }` → `pub struct SqlStorageCursorNextReturnBuilder<T> { inner: SqlStorageCursorNextReturn<T> }`

### `SubtleCrypto`

- `pub async fn decrypt(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, cipher_text: &ArrayBuffer) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn decrypt(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, cipher_text: &ArrayBuffer) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn decrypt_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, cipher_text: &ArrayBuffer) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn decrypt_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, cipher_text: &ArrayBuffer) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn decrypt_with_str_and_typed_array(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, cipher_text: &Object) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn decrypt_with_str_and_typed_array(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, cipher_text: &Object) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn decrypt_with_str_and_typed_array_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, cipher_text: &Object) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn decrypt_with_str_and_typed_array_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, cipher_text: &Object) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn derive_bits(this: &SubtleCrypto, algorithm: &str, base_key: &CryptoKey) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn derive_bits(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn derive_bits_promise(this: &SubtleCrypto, algorithm: &str, base_key: &CryptoKey) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn derive_bits_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn derive_bits_with_str_and_length(this: &SubtleCrypto, algorithm: &str, base_key: &CryptoKey, length: &JsNullable<Number>) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn derive_bits_with_str_and_length(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, length: &JsNullable<Number>) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn derive_bits_with_str_and_length_promise(this: &SubtleCrypto, algorithm: &str, base_key: &CryptoKey, length: &JsNullable<Number>) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn derive_bits_with_str_and_length_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, length: &JsNullable<Number>) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn derive_key(this: &SubtleCrypto, algorithm: &str, base_key: &CryptoKey, derived_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn derive_key(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn derive_key_promise(this: &SubtleCrypto, algorithm: &str, base_key: &CryptoKey, derived_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn derive_key_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn derive_key_with_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, algorithm: &str, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn derive_key_with_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn derive_key_with_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, algorithm: &str, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn derive_key_with_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn derive_key_with_subtle_crypto_derive_key_algorithm_and_str(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn derive_key_with_subtle_crypto_derive_key_algorithm_and_str(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn derive_key_with_subtle_crypto_derive_key_algorithm_and_str_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn derive_key_with_subtle_crypto_derive_key_algorithm_and_str_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn derive_key_with_subtle_crypto_derive_key_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn derive_key_with_subtle_crypto_derive_key_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn derive_key_with_subtle_crypto_derive_key_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn derive_key_with_subtle_crypto_derive_key_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn digest(this: &SubtleCrypto, algorithm: &str, data: &ArrayBuffer) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn digest(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, data: &ArrayBuffer) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn digest_promise(this: &SubtleCrypto, algorithm: &str, data: &ArrayBuffer) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn digest_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, data: &ArrayBuffer) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn digest_with_str_and_typed_array(this: &SubtleCrypto, algorithm: &str, data: &Object) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn digest_with_str_and_typed_array(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, data: &Object) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn digest_with_str_and_typed_array_promise(this: &SubtleCrypto, algorithm: &str, data: &Object) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn digest_with_str_and_typed_array_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, data: &Object) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn encrypt(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, plain_text: &ArrayBuffer) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn encrypt(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, plain_text: &ArrayBuffer) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn encrypt_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, plain_text: &ArrayBuffer) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn encrypt_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, plain_text: &ArrayBuffer) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn encrypt_with_str_and_typed_array(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, plain_text: &Object) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn encrypt_with_str_and_typed_array(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, plain_text: &Object) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn encrypt_with_str_and_typed_array_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, plain_text: &Object) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn encrypt_with_str_and_typed_array_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, plain_text: &Object) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn export_key(this: &SubtleCrypto, format: &str, key: &CryptoKey) -> Result<ExportKeyReturnKind, crate::Error>;` → `pub async fn export_key(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey) -> Result<ExportKeyReturnKind, crate::Error>;`
- `pub fn export_key_promise(this: &SubtleCrypto, format: &str, key: &CryptoKey) -> Result<Promise, crate::Error>;` → `pub fn export_key_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey) -> Result<Promise, crate::Error>;`
- `pub async fn generate_key(this: &SubtleCrypto, algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<GenerateKeyReturnKind, crate::Error>;` → `pub async fn generate_key(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<GenerateKeyReturnKind, crate::Error>;`
- `pub fn generate_key_promise(this: &SubtleCrypto, algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise, crate::Error>;` → `pub fn generate_key_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise, crate::Error>;`
- `pub async fn generate_key_with_subtle_crypto_generate_key_algorithm(this: &SubtleCrypto, algorithm: &SubtleCryptoGenerateKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<GenerateKeyReturnKind, crate::Error>;` → `pub async fn generate_key_with_subtle_crypto_generate_key_algorithm(this: &SubtleCrypto, algorithm: &SubtleCryptoGenerateKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<GenerateKeyReturnKind, crate::Error>;`
- `pub fn generate_key_with_subtle_crypto_generate_key_algorithm_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoGenerateKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise, crate::Error>;` → `pub fn generate_key_with_subtle_crypto_generate_key_algorithm_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoGenerateKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise, crate::Error>;`
- `pub async fn import_key(this: &SubtleCrypto, format: &str, key_data: &ArrayBuffer, algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_promise(this: &SubtleCrypto, format: &str, key_data: &ArrayBuffer, algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_array_buffer_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: &str, key_data: &ArrayBuffer, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_array_buffer_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_array_buffer_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: &str, key_data: &ArrayBuffer, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_array_buffer_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_json_web_key_and_str(this: &SubtleCrypto, format: &str, key_data: &JsonWebKey, algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_json_web_key_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_json_web_key_and_str_promise(this: &SubtleCrypto, format: &str, key_data: &JsonWebKey, algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_json_web_key_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_json_web_key_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: &str, key_data: &JsonWebKey, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_json_web_key_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_json_web_key_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: &str, key_data: &JsonWebKey, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_json_web_key_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_typed_array_and_str(this: &SubtleCrypto, format: &str, key_data: &Object, algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_typed_array_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_typed_array_and_str_promise(this: &SubtleCrypto, format: &str, key_data: &Object, algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_typed_array_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_typed_array_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: &str, key_data: &Object, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_typed_array_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_typed_array_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: &str, key_data: &Object, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_typed_array_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn sign(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, data: &ArrayBuffer) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn sign(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, data: &ArrayBuffer) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn sign_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, data: &ArrayBuffer) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn sign_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, data: &ArrayBuffer) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn sign_with_str_and_typed_array(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, data: &Object) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn sign_with_str_and_typed_array(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, data: &Object) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn sign_with_str_and_typed_array_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, data: &Object) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn sign_with_str_and_typed_array_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, data: &Object) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn unwrap_key(this: &SubtleCrypto, format: &str, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &str, unwrapped_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_promise(this: &SubtleCrypto, format: &str, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &str, unwrapped_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_array_buffer_and_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: &str, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &str, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_array_buffer_and_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_array_buffer_and_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: &str, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &str, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_array_buffer_and_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_str(this: &SubtleCrypto, format: &str, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_str_promise(this: &SubtleCrypto, format: &str, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: &str, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: &str, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_typed_array_and_str_and_str(this: &SubtleCrypto, format: &str, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &str, unwrapped_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_typed_array_and_str_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_typed_array_and_str_and_str_promise(this: &SubtleCrypto, format: &str, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &str, unwrapped_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_typed_array_and_str_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_typed_array_and_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: &str, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &str, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_typed_array_and_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_typed_array_and_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: &str, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &str, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_typed_array_and_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_str(this: &SubtleCrypto, format: &str, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_str_promise(this: &SubtleCrypto, format: &str, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &str, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: &str, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: &str, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn verify(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, signature: &ArrayBuffer, data: &ArrayBuffer) -> Result<bool, crate::Error>;` → `pub async fn verify(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, signature: &ArrayBuffer, data: &ArrayBuffer) -> Result<bool, crate::Error>;`
- `pub fn verify_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, signature: &ArrayBuffer, data: &ArrayBuffer) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn verify_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, signature: &ArrayBuffer, data: &ArrayBuffer) -> Result<Promise<Boolean>, crate::Error>;`
- `pub async fn verify_with_str_and_array_buffer_and_typed_array(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, signature: &ArrayBuffer, data: &Object) -> Result<bool, crate::Error>;` → `pub async fn verify_with_str_and_array_buffer_and_typed_array(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, signature: &ArrayBuffer, data: &Object) -> Result<bool, crate::Error>;`
- `pub fn verify_with_str_and_array_buffer_and_typed_array_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, signature: &ArrayBuffer, data: &Object) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn verify_with_str_and_array_buffer_and_typed_array_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, signature: &ArrayBuffer, data: &Object) -> Result<Promise<Boolean>, crate::Error>;`
- `pub async fn verify_with_str_and_typed_array_and_array_buffer(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, signature: &Object, data: &ArrayBuffer) -> Result<bool, crate::Error>;` → `pub async fn verify_with_str_and_typed_array_and_array_buffer(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, signature: &Object, data: &ArrayBuffer) -> Result<bool, crate::Error>;`
- `pub fn verify_with_str_and_typed_array_and_array_buffer_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, signature: &Object, data: &ArrayBuffer) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn verify_with_str_and_typed_array_and_array_buffer_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, signature: &Object, data: &ArrayBuffer) -> Result<Promise<Boolean>, crate::Error>;`
- `pub async fn verify_with_str_and_typed_array_and_typed_array(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, signature: &Object, data: &Object) -> Result<bool, crate::Error>;` → `pub async fn verify_with_str_and_typed_array_and_typed_array(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, signature: &Object, data: &Object) -> Result<bool, crate::Error>;`
- `pub fn verify_with_str_and_typed_array_and_typed_array_promise(this: &SubtleCrypto, algorithm: &str, key: &CryptoKey, signature: &Object, data: &Object) -> Result<Promise<Boolean>, crate::Error>;` → `pub fn verify_with_str_and_typed_array_and_typed_array_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, signature: &Object, data: &Object) -> Result<Promise<Boolean>, crate::Error>;`
- `pub async fn wrap_key(this: &SubtleCrypto, format: &str, key: &CryptoKey, wrapping_key: &CryptoKey, wrap_algorithm: &str) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn wrap_key(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, wrapping_key: &CryptoKey, wrap_algorithm: impl ::wasm_bindgen::JsStringLike) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn wrap_key_promise(this: &SubtleCrypto, format: &str, key: &CryptoKey, wrapping_key: &CryptoKey, wrap_algorithm: &str) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn wrap_key_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, wrapping_key: &CryptoKey, wrap_algorithm: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<ArrayBuffer>, crate::Error>;`
- `pub async fn wrap_key_with_subtle_crypto_encrypt_algorithm(this: &SubtleCrypto, format: &str, key: &CryptoKey, wrapping_key: &CryptoKey, wrap_algorithm: &SubtleCryptoEncryptAlgorithm) -> Result<ArrayBuffer, crate::Error>;` → `pub async fn wrap_key_with_subtle_crypto_encrypt_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, wrapping_key: &CryptoKey, wrap_algorithm: &SubtleCryptoEncryptAlgorithm) -> Result<ArrayBuffer, crate::Error>;`
- `pub fn wrap_key_with_subtle_crypto_encrypt_algorithm_promise(this: &SubtleCrypto, format: &str, key: &CryptoKey, wrapping_key: &CryptoKey, wrap_algorithm: &SubtleCryptoEncryptAlgorithm) -> Result<Promise<ArrayBuffer>, crate::Error>;` → `pub fn wrap_key_with_subtle_crypto_encrypt_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key: &CryptoKey, wrapping_key: &CryptoKey, wrap_algorithm: &SubtleCryptoEncryptAlgorithm) -> Result<Promise<ArrayBuffer>, crate::Error>;`

### `SubtleCryptoDeriveKeyAlgorithm`

- `pub fn builder(name: &str) -> SubtleCryptoDeriveKeyAlgorithmBuilder;` → `pub fn builder(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoDeriveKeyAlgorithmBuilder;`
- added → `pub fn name_js_string(this: &SubtleCryptoDeriveKeyAlgorithm) -> JsString;`
- `pub fn new(name: &str) -> SubtleCryptoDeriveKeyAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoDeriveKeyAlgorithm;`
- `pub fn set_hash(this: &SubtleCryptoDeriveKeyAlgorithm, val: &str);` → `pub fn set_hash(this: &SubtleCryptoDeriveKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &SubtleCryptoDeriveKeyAlgorithm, val: &str);` → `pub fn set_name(this: &SubtleCryptoDeriveKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `SubtleCryptoDeriveKeyAlgorithmBuilder`

- `pub fn hash(self, val: &str) -> Self;` → `pub fn hash(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn info_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn info_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn salt_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn salt_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`

### `SubtleCryptoEncryptAlgorithm`

- `pub fn builder(name: &str) -> SubtleCryptoEncryptAlgorithmBuilder;` → `pub fn builder(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoEncryptAlgorithmBuilder;`
- added → `pub fn name_js_string(this: &SubtleCryptoEncryptAlgorithm) -> JsString;`
- `pub fn new(name: &str) -> SubtleCryptoEncryptAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoEncryptAlgorithm;`
- `pub fn set_name(this: &SubtleCryptoEncryptAlgorithm, val: &str);` → `pub fn set_name(this: &SubtleCryptoEncryptAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `SubtleCryptoEncryptAlgorithmBuilder`

- `pub fn additional_data_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn additional_data_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn counter_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn counter_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn iv_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn iv_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`
- `pub fn label_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn label_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`

### `SubtleCryptoGenerateKeyAlgorithm`

- `pub fn builder(name: &str) -> SubtleCryptoGenerateKeyAlgorithmBuilder;` → `pub fn builder(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoGenerateKeyAlgorithmBuilder;`
- added → `pub fn name_js_string(this: &SubtleCryptoGenerateKeyAlgorithm) -> JsString;`
- added → `pub fn named_curve_js_string(this: &SubtleCryptoGenerateKeyAlgorithm) -> Option<JsString>;`
- `pub fn new(name: &str) -> SubtleCryptoGenerateKeyAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoGenerateKeyAlgorithm;`
- `pub fn set_hash(this: &SubtleCryptoGenerateKeyAlgorithm, val: &str);` → `pub fn set_hash(this: &SubtleCryptoGenerateKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &SubtleCryptoGenerateKeyAlgorithm, val: &str);` → `pub fn set_name(this: &SubtleCryptoGenerateKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_named_curve(this: &SubtleCryptoGenerateKeyAlgorithm, val: &str);` → `pub fn set_named_curve(this: &SubtleCryptoGenerateKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `SubtleCryptoGenerateKeyAlgorithmBuilder`

- `pub fn hash(self, val: &str) -> Self;` → `pub fn hash(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn named_curve(self, val: &str) -> Self;` → `pub fn named_curve(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn public_exponent_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self;` → `pub fn public_exponent_with_typed_array<T: ::js_sys::TypedArray>(self, val: &T) -> Self where for<'__wbg> &'__wbg T: ::wasm_bindgen::convert::IntoWasmAbi;`

### `SubtleCryptoHashAlgorithm`

- added → `pub fn name_js_string(this: &SubtleCryptoHashAlgorithm) -> JsString;`
- `pub fn new(name: &str) -> SubtleCryptoHashAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoHashAlgorithm;`
- `pub fn set_name(this: &SubtleCryptoHashAlgorithm, val: &str);` → `pub fn set_name(this: &SubtleCryptoHashAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `SubtleCryptoImportKeyAlgorithm`

- `pub fn builder(name: &str) -> SubtleCryptoImportKeyAlgorithmBuilder;` → `pub fn builder(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoImportKeyAlgorithmBuilder;`
- added → `pub fn name_js_string(this: &SubtleCryptoImportKeyAlgorithm) -> JsString;`
- added → `pub fn named_curve_js_string(this: &SubtleCryptoImportKeyAlgorithm) -> Option<JsString>;`
- `pub fn new(name: &str) -> SubtleCryptoImportKeyAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoImportKeyAlgorithm;`
- `pub fn set_hash(this: &SubtleCryptoImportKeyAlgorithm, val: &str);` → `pub fn set_hash(this: &SubtleCryptoImportKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &SubtleCryptoImportKeyAlgorithm, val: &str);` → `pub fn set_name(this: &SubtleCryptoImportKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_named_curve(this: &SubtleCryptoImportKeyAlgorithm, val: &str);` → `pub fn set_named_curve(this: &SubtleCryptoImportKeyAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `SubtleCryptoImportKeyAlgorithmBuilder`

- `pub fn hash(self, val: &str) -> Self;` → `pub fn hash(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn named_curve(self, val: &str) -> Self;` → `pub fn named_curve(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `SubtleCryptoSignAlgorithm`

- `pub fn builder(name: &str) -> SubtleCryptoSignAlgorithmBuilder;` → `pub fn builder(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoSignAlgorithmBuilder;`
- added → `pub fn name_js_string(this: &SubtleCryptoSignAlgorithm) -> JsString;`
- `pub fn new(name: &str) -> SubtleCryptoSignAlgorithm;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike) -> SubtleCryptoSignAlgorithm;`
- `pub fn set_hash(this: &SubtleCryptoSignAlgorithm, val: &str);` → `pub fn set_hash(this: &SubtleCryptoSignAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &SubtleCryptoSignAlgorithm, val: &str);` → `pub fn set_name(this: &SubtleCryptoSignAlgorithm, val: impl ::wasm_bindgen::JsStringLike);`

### `SubtleCryptoSignAlgorithmBuilder`

- `pub fn hash(self, val: &str) -> Self;` → `pub fn hash(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `SyncKvListOptions`

- added → `pub fn end_js_string(this: &SyncKvListOptions) -> Option<JsString>;`
- added → `pub fn prefix_js_string(this: &SyncKvListOptions) -> Option<JsString>;`
- `pub fn set_end(this: &SyncKvListOptions, val: &str);` → `pub fn set_end(this: &SyncKvListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_prefix(this: &SyncKvListOptions, val: &str);` → `pub fn set_prefix(this: &SyncKvListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_start(this: &SyncKvListOptions, val: &str);` → `pub fn set_start(this: &SyncKvListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_start_after(this: &SyncKvListOptions, val: &str);` → `pub fn set_start_after(this: &SyncKvListOptions, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn start_after_js_string(this: &SyncKvListOptions) -> Option<JsString>;`
- added → `pub fn start_js_string(this: &SyncKvListOptions) -> Option<JsString>;`

### `SyncKvListOptionsBuilder`

- `pub fn end(self, val: &str) -> Self;` → `pub fn end(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn prefix(self, val: &str) -> Self;` → `pub fn prefix(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn start(self, val: &str) -> Self;` → `pub fn start(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn start_after(self, val: &str) -> Self;` → `pub fn start_after(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `SyncKvStorage`

- `pub fn delete(this: &SyncKvStorage, key: &str) -> Result<bool, crate::Error>;` → `pub fn delete(this: &SyncKvStorage, key: impl ::wasm_bindgen::JsStringLike) -> Result<bool, crate::Error>;`
- `pub fn get<T: ::wasm_bindgen::JsGeneric>(this: &SyncKvStorage, key: &str) -> Result<Option<T>, crate::Error>;` → `pub fn get<T>(this: &SyncKvStorage, key: impl ::wasm_bindgen::JsStringLike) -> Result<Option<T>, crate::Error>;`
- `pub fn put<T: ::wasm_bindgen::JsGeneric>(this: &SyncKvStorage, key: &str, value: &T) -> Result<(), crate::Error>;` → `pub fn put<T>(this: &SyncKvStorage, key: impl ::wasm_bindgen::JsStringLike, value: T) -> Result<(), crate::Error>;`

### `SyncKvStorageList`

- `pub fn iterator<T: ::wasm_bindgen::JsGeneric>(this: &SyncKvStorageList<T>) -> Result<Iterator<ArrayTuple<(JsString, T)>>, crate::Error>;` → `pub fn iterator<T>(this: &SyncKvStorageList<T>) -> Result<Iterator<ArrayTuple<(JsString, T)>>, crate::Error>;`
- `pub type SyncKvStorageList<T: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type SyncKvStorageList<T = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `TlsOptions`

- added → `pub fn expected_server_hostname_js_string(this: &TlsOptions) -> Option<JsString>;`
- `pub fn set_expected_server_hostname(this: &TlsOptions, val: &str);` → `pub fn set_expected_server_hostname(this: &TlsOptions, val: impl ::wasm_bindgen::JsStringLike);`

### `TlsOptionsBuilder`

- `pub fn expected_server_hostname(self, val: &str) -> Self;` → `pub fn expected_server_hostname(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `TraceDiagnosticChannelEvent`

- added → `pub fn channel_js_string(this: &TraceDiagnosticChannelEvent) -> JsString;`

### `TraceException`

- added → `pub fn message_js_string(this: &TraceException) -> JsString;`
- added → `pub fn name_js_string(this: &TraceException) -> JsString;`
- added → `pub fn stack_js_string(this: &TraceException) -> Option<JsString>;`

### `TraceItem`

- added → `pub fn dispatch_namespace_js_string(this: &TraceItem) -> Option<JsString>;`
- added → `pub fn durable_object_id_js_string(this: &TraceItem) -> Option<JsString>;`
- added → `pub fn entrypoint_js_string(this: &TraceItem) -> Option<JsString>;`
- added → `pub fn execution_model_js_string(this: &TraceItem) -> JsString;`
- added → `pub fn outcome_js_string(this: &TraceItem) -> JsString;`
- added → `pub fn script_name_js_string(this: &TraceItem) -> Option<JsString>;`
- added → `pub fn script_tags_js_string(this: &TraceItem) -> Option<Array<JsString>>;`

### `TraceItemEmailEventInfo`

- added → `pub fn mail_from_js_string(this: &TraceItemEmailEventInfo) -> JsString;`
- added → `pub fn rcpt_to_js_string(this: &TraceItemEmailEventInfo) -> JsString;`

### `TraceItemFetchEventInfoRequest`

- added → `pub fn method_js_string(this: &TraceItemFetchEventInfoRequest) -> JsString;`
- added → `pub fn url_js_string(this: &TraceItemFetchEventInfoRequest) -> JsString;`

### `TraceItemHibernatableWebSocketEventInfoClose`

- added → `pub fn web_socket_event_type_js_string(this: &TraceItemHibernatableWebSocketEventInfoClose) -> JsString;`

### `TraceItemHibernatableWebSocketEventInfoError`

- added → `pub fn web_socket_event_type_js_string(this: &TraceItemHibernatableWebSocketEventInfoError) -> JsString;`

### `TraceItemHibernatableWebSocketEventInfoMessage`

- added → `pub fn web_socket_event_type_js_string(this: &TraceItemHibernatableWebSocketEventInfoMessage) -> JsString;`

### `TraceItemJsRpcEventInfo`

- added → `pub fn rpc_method_js_string(this: &TraceItemJsRpcEventInfo) -> JsString;`

### `TraceItemQueueEventInfo`

- added → `pub fn queue_js_string(this: &TraceItemQueueEventInfo) -> JsString;`

### `TraceItemScheduledEventInfo`

- added → `pub fn cron_js_string(this: &TraceItemScheduledEventInfo) -> JsString;`

### `TraceItemTailEventInfoTailItem`

- added → `pub fn script_name_js_string(this: &TraceItemTailEventInfoTailItem) -> Option<JsString>;`

### `TraceLog`

- added → `pub fn level_js_string(this: &TraceLog) -> JsString;`

### `TraceLogErrorInfo`

- `pub fn builder(name: &str, message: &str) -> TraceLogErrorInfoBuilder;` → `pub fn builder(name: impl ::wasm_bindgen::JsStringLike, message: impl ::wasm_bindgen::JsStringLike) -> TraceLogErrorInfoBuilder;`
- added → `pub fn message_js_string(this: &TraceLogErrorInfo) -> JsString;`
- added → `pub fn name_js_string(this: &TraceLogErrorInfo) -> JsString;`
- `pub fn new(name: &str, message: &str) -> TraceLogErrorInfo;` → `pub fn new(name: impl ::wasm_bindgen::JsStringLike, message: impl ::wasm_bindgen::JsStringLike) -> TraceLogErrorInfo;`
- `pub fn set_message(this: &TraceLogErrorInfo, val: &str);` → `pub fn set_message(this: &TraceLogErrorInfo, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &TraceLogErrorInfo, val: &str);` → `pub fn set_name(this: &TraceLogErrorInfo, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_stack(this: &TraceLogErrorInfo, val: &str);` → `pub fn set_stack(this: &TraceLogErrorInfo, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn stack_js_string(this: &TraceLogErrorInfo) -> Option<JsString>;`

### `TraceLogErrorInfoBuilder`

- `pub fn stack(self, val: &str) -> Self;` → `pub fn stack(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`

### `TracePreviewInfo`

- added → `pub fn id_js_string(this: &TracePreviewInfo) -> JsString;`
- added → `pub fn name_js_string(this: &TracePreviewInfo) -> JsString;`
- `pub fn new(id: &str, slug: &str, name: &str) -> TracePreviewInfo;` → `pub fn new(id: impl ::wasm_bindgen::JsStringLike, slug: impl ::wasm_bindgen::JsStringLike, name: impl ::wasm_bindgen::JsStringLike) -> TracePreviewInfo;`
- `pub fn set_id(this: &TracePreviewInfo, val: &str);` → `pub fn set_id(this: &TracePreviewInfo, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_name(this: &TracePreviewInfo, val: &str);` → `pub fn set_name(this: &TracePreviewInfo, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_slug(this: &TracePreviewInfo, val: &str);` → `pub fn set_slug(this: &TracePreviewInfo, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn slug_js_string(this: &TracePreviewInfo) -> JsString;`

### `Tracing`

- `pub fn enter_span<T: ::wasm_bindgen::JsGeneric>(this: &Tracing, name: &str, callback: &Function, args: &[JsValue]) -> Result<T, crate::Error>; [variadic]` → `pub fn enter_span<T>(this: &Tracing, name: impl ::wasm_bindgen::JsStringLike, callback: &Function, args: &[JsValue]) -> Result<T, crate::Error>; [variadic]`
- `pub fn start_active_span<T: ::wasm_bindgen::JsGeneric>(this: &Tracing, name: &str, callback: &Function, args: &[JsValue]) -> Result<T, crate::Error>; [variadic]` → `pub fn start_active_span<T>(this: &Tracing, name: impl ::wasm_bindgen::JsStringLike, callback: &Function, args: &[JsValue]) -> Result<T, crate::Error>; [variadic]`
- `pub fn start_span(this: &Tracing, name: &str) -> Result<Span, crate::Error>;` → `pub fn start_span(this: &Tracing, name: impl ::wasm_bindgen::JsStringLike) -> Result<Span, crate::Error>;`

### `TransformStream`

- `pub fn new() -> Result<TransformStream, crate::Error>;` → `pub fn new<I, O>() -> Result<TransformStream<I, O>, crate::Error>;`
- `pub fn new_with_transformer<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(transformer: &Transformer<I, O>) -> Result<TransformStream<I, O>, crate::Error>;` → `pub fn new_with_transformer<I, O>(transformer: &Transformer<I, O>) -> Result<TransformStream<I, O>, crate::Error>;`
- `pub fn new_with_transformer_and_writable_strategy<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(transformer: &Transformer<I, O>, writable_strategy: &QueuingStrategy<I>) -> Result<TransformStream<I, O>, crate::Error>;` → `pub fn new_with_transformer_and_writable_strategy<I, O>(transformer: &Transformer<I, O>, writable_strategy: &QueuingStrategy<I>) -> Result<TransformStream<I, O>, crate::Error>;`
- `pub fn new_with_transformer_and_writable_strategy_and_readable_strategy<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(transformer: &Transformer<I, O>, writable_strategy: &QueuingStrategy<I>, readable_strategy: &QueuingStrategy<O>) -> Result<TransformStream<I, O>, crate::Error>;` → `pub fn new_with_transformer_and_writable_strategy_and_readable_strategy<I, O>(transformer: &Transformer<I, O>, writable_strategy: &QueuingStrategy<I>, readable_strategy: &QueuingStrategy<O>) -> Result<TransformStream<I, O>, crate::Error>;`
- `pub fn readable<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &TransformStream<I, O>) -> ReadableStream<O>;` → `pub fn readable<I, O>(this: &TransformStream<I, O>) -> ReadableStream<O>;`
- `pub fn writable<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &TransformStream<I, O>) -> WritableStream<I>;` → `pub fn writable<I, O>(this: &TransformStream<I, O>) -> WritableStream<I>;`
- `pub type TransformStream<I: ::wasm_bindgen::JsGeneric = JsValue, O: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type TransformStream<I = JsValue, O = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `TransformStreamDefaultController`

- `pub fn desired_size<O: ::wasm_bindgen::JsGeneric>(this: &TransformStreamDefaultController<O>) -> Option<f64>;` → `pub fn desired_size<O>(this: &TransformStreamDefaultController<O>) -> Option<f64>;`
- `pub fn enqueue<O: ::wasm_bindgen::JsGeneric>(this: &TransformStreamDefaultController<O>) -> Result<(), crate::Error>;` → `pub fn enqueue<O>(this: &TransformStreamDefaultController<O>) -> Result<(), crate::Error>;`
- `pub fn enqueue_with_chunk<O: ::wasm_bindgen::JsGeneric>(this: &TransformStreamDefaultController<O>, chunk: &O) -> Result<(), crate::Error>;` → `pub fn enqueue_with_chunk<O>(this: &TransformStreamDefaultController<O>, chunk: O) -> Result<(), crate::Error>;`
- `pub fn error<O: ::wasm_bindgen::JsGeneric>(this: &TransformStreamDefaultController<O>, reason: &JsValue) -> Result<(), crate::Error>;` → `pub fn error<O>(this: &TransformStreamDefaultController<O>, reason: &JsValue) -> Result<(), crate::Error>;`
- `pub fn terminate<O: ::wasm_bindgen::JsGeneric>(this: &TransformStreamDefaultController<O>) -> Result<(), crate::Error>;` → `pub fn terminate<O>(this: &TransformStreamDefaultController<O>) -> Result<(), crate::Error>;`
- `pub type TransformStreamDefaultController<O: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type TransformStreamDefaultController<O = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `Transformer`

- `pub fn cancel<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>) -> Option<Function<fn(JsValue) -> JsOption<Promise<Undefined>>>>;` → `pub fn cancel<I, O>(this: &Transformer<I, O>) -> Option<Function<fn(JsValue) -> JsOption<Promise<Undefined>>>>;`
- `pub fn expected_length<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>) -> Option<f64>;` → `pub fn expected_length<I, O>(this: &Transformer<I, O>) -> Option<f64>;`
- `pub fn flush<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>) -> Option<Function<fn(TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>>;` → `pub fn flush<I, O>(this: &Transformer<I, O>) -> Option<Function<fn(TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn readable_type<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>) -> Option<String>;` → `pub fn readable_type<I, O>(this: &Transformer<I, O>) -> Option<String>;`
- added → `pub fn readable_type_js_string<I, O>(this: &Transformer<I, O>) -> Option<JsString>;`
- `pub fn set_cancel<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>, val: &Function<fn(JsValue) -> JsOption<Promise<Undefined>>>);` → `pub fn set_cancel<I, O>(this: &Transformer<I, O>, val: &Function<fn(JsValue) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_expected_length<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>, val: f64);` → `pub fn set_expected_length<I, O>(this: &Transformer<I, O>, val: f64);`
- `pub fn set_flush<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>, val: &Function<fn(TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_flush<I, O>(this: &Transformer<I, O>, val: &Function<fn(TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_readable_type<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>, val: &str);` → `pub fn set_readable_type<I, O>(this: &Transformer<I, O>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_start<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>, val: &Function<fn(TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_start<I, O>(this: &Transformer<I, O>, val: &Function<fn(TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_transform<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>, val: &Function<fn(I, TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_transform<I, O>(this: &Transformer<I, O>, val: &Function<fn(I, TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_writable_type<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>, val: &str);` → `pub fn set_writable_type<I, O>(this: &Transformer<I, O>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn start<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>) -> Option<Function<fn(TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>>;` → `pub fn start<I, O>(this: &Transformer<I, O>) -> Option<Function<fn(TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn transform<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>) -> Option<Function<fn(I, TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>>;` → `pub fn transform<I, O>(this: &Transformer<I, O>) -> Option<Function<fn(I, TransformStreamDefaultController<O>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn writable_type<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric>(this: &Transformer<I, O>) -> Option<String>;` → `pub fn writable_type<I, O>(this: &Transformer<I, O>) -> Option<String>;`
- added → `pub fn writable_type_js_string<I, O>(this: &Transformer<I, O>) -> Option<JsString>;`
- `impl<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric> Transformer<I, O>` → `impl<I: ::wasm_bindgen::convert::IntoWasmAbi, O: ::wasm_bindgen::convert::IntoWasmAbi> Transformer<I, O>`
- `pub type Transformer<I: ::wasm_bindgen::JsGeneric = JsValue, O: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type Transformer<I = JsValue, O = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `TransformerBuilder`

- `pub fn readable_type(self, val: &str) -> Self;` → `pub fn readable_type(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `pub fn writable_type(self, val: &str) -> Self;` → `pub fn writable_type(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `impl<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric> From<Transformer<I, O>> for TransformerBuilder<I, O>` → `impl<I: ::wasm_bindgen::convert::IntoWasmAbi, O: ::wasm_bindgen::convert::IntoWasmAbi> From<Transformer<I, O>> for TransformerBuilder<I, O>`
- `impl<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric> TransformerBuilder<I, O>` → `impl<I: ::wasm_bindgen::convert::IntoWasmAbi, O: ::wasm_bindgen::convert::IntoWasmAbi> TransformerBuilder<I, O>`
- `pub struct TransformerBuilder<I: ::wasm_bindgen::JsGeneric, O: ::wasm_bindgen::JsGeneric> { inner: Transformer<I, O> }` → `pub struct TransformerBuilder<I, O> { inner: Transformer<I, O> }`

### `URL`

- `pub fn can_parse(url: &str) -> Result<bool, crate::Error>;` → `pub fn can_parse(url: impl ::wasm_bindgen::JsStringLike) -> Result<bool, crate::Error>;`
- `pub fn can_parse_with_base(url: &str, base: &str) -> Result<bool, crate::Error>;` → `pub fn can_parse_with_base(url: impl ::wasm_bindgen::JsStringLike, base: impl ::wasm_bindgen::JsStringLike) -> Result<bool, crate::Error>;`
- added → `pub fn create_object_url_js_string(object: &File) -> Result<JsString, crate::Error>;`
- added → `pub fn create_object_url_with_blob_js_string(object: &Blob) -> Result<JsString, crate::Error>;`
- added → `pub fn hash_js_string(this: &URL) -> JsString;`
- added → `pub fn host_js_string(this: &URL) -> JsString;`
- added → `pub fn hostname_js_string(this: &URL) -> JsString;`
- added → `pub fn href_js_string(this: &URL) -> JsString;`
- `pub fn new(url: &str) -> Result<URL, crate::Error>;` → `pub fn new(url: impl ::wasm_bindgen::JsStringLike) -> Result<URL, crate::Error>;`
- `pub fn new_with_str_and_str(url: &str, base: &str) -> Result<URL, crate::Error>;` → `pub fn new_with_str_and_str(url: impl ::wasm_bindgen::JsStringLike, base: impl ::wasm_bindgen::JsStringLike) -> Result<URL, crate::Error>;`
- `pub fn new_with_str_and_url(url: &str, base: &URL) -> Result<URL, crate::Error>;` → `pub fn new_with_str_and_url(url: impl ::wasm_bindgen::JsStringLike, base: &URL) -> Result<URL, crate::Error>;`
- `pub fn new_with_url_and_str(url: &URL, base: &str) -> Result<URL, crate::Error>;` → `pub fn new_with_url_and_str(url: &URL, base: impl ::wasm_bindgen::JsStringLike) -> Result<URL, crate::Error>;`
- added → `pub fn origin_js_string(this: &URL) -> JsString;`
- `pub fn parse(url: &str) -> Result<Option<URL>, crate::Error>;` → `pub fn parse(url: impl ::wasm_bindgen::JsStringLike) -> Result<Option<URL>, crate::Error>;`
- `pub fn parse_with_base(url: &str, base: &str) -> Result<Option<URL>, crate::Error>;` → `pub fn parse_with_base(url: impl ::wasm_bindgen::JsStringLike, base: impl ::wasm_bindgen::JsStringLike) -> Result<Option<URL>, crate::Error>;`
- added → `pub fn password_js_string(this: &URL) -> JsString;`
- added → `pub fn pathname_js_string(this: &URL) -> JsString;`
- added → `pub fn port_js_string(this: &URL) -> JsString;`
- added → `pub fn protocol_js_string(this: &URL) -> JsString;`
- `pub fn revoke_object_url(object_url: &str) -> Result<(), crate::Error>;` → `pub fn revoke_object_url(object_url: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- added → `pub fn search_js_string(this: &URL) -> JsString;`
- `pub fn set_hash(this: &URL, val: &str);` → `pub fn set_hash(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_host(this: &URL, val: &str);` → `pub fn set_host(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_hostname(this: &URL, val: &str);` → `pub fn set_hostname(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_href(this: &URL, val: &str);` → `pub fn set_href(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_password(this: &URL, val: &str);` → `pub fn set_password(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_pathname(this: &URL, val: &str);` → `pub fn set_pathname(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_port(this: &URL, val: &str);` → `pub fn set_port(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_protocol(this: &URL, val: &str);` → `pub fn set_protocol(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_search(this: &URL, val: &str);` → `pub fn set_search(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_username(this: &URL, val: &str);` → `pub fn set_username(this: &URL, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn to_json_js_string(this: &URL) -> Result<JsString, crate::Error>;`
- added → `pub fn to_string_js_string(this: &URL) -> Result<JsString, crate::Error>;`
- added → `pub fn username_js_string(this: &URL) -> JsString;`

### `URLSearchParams`

- `pub fn append(this: &URLSearchParams, name: &str, value: &str) -> Result<(), crate::Error>;` → `pub fn append(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn delete(this: &URLSearchParams, name: &str) -> Result<(), crate::Error>;` → `pub fn delete(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn delete_with_value(this: &URLSearchParams, name: &str, value: &str) -> Result<(), crate::Error>;` → `pub fn delete_with_value(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- `pub fn for_each_with_this_arg<This: ::wasm_bindgen::JsGeneric>(this: &URLSearchParams, callback: &Function<fn(JsString, JsString, URLSearchParams) -> Undefined>, this_arg: &This) -> Result<(), crate::Error>;` → `pub fn for_each_with_this_arg<This>(this: &URLSearchParams, callback: &Function<fn(JsString, JsString, URLSearchParams) -> Undefined>, this_arg: This) -> Result<(), crate::Error>;`
- `pub fn get(this: &URLSearchParams, name: &str) -> Result<Option<String>, crate::Error>;` → `pub fn get(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike) -> Result<Option<String>, crate::Error>;`
- `pub fn get_all(this: &URLSearchParams, name: &str) -> Result<Vec<String>, crate::Error>;` → `pub fn get_all(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike) -> Result<Vec<String>, crate::Error>;`
- added → `pub fn get_all_js_string(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike) -> Result<Array<JsString>, crate::Error>;`
- added → `pub fn get_js_string(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike) -> Result<Option<JsString>, crate::Error>;`
- `pub fn has(this: &URLSearchParams, name: &str) -> Result<bool, crate::Error>;` → `pub fn has(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike) -> Result<bool, crate::Error>;`
- `pub fn has_with_value(this: &URLSearchParams, name: &str, value: &str) -> Result<bool, crate::Error>;` → `pub fn has_with_value(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike) -> Result<bool, crate::Error>;`
- `pub fn new_with_str(init: &str) -> Result<URLSearchParams, crate::Error>;` → `pub fn new_with_str(init: impl ::wasm_bindgen::JsStringLike) -> Result<URLSearchParams, crate::Error>;`
- `pub fn set(this: &URLSearchParams, name: &str, value: &str) -> Result<(), crate::Error>;` → `pub fn set(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike, value: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- added → `pub fn to_string_js_string(this: &URLSearchParams) -> Result<JsString, crate::Error>;`

### `UnderlyingByteSource`

- `pub fn set_type(this: &UnderlyingByteSource, val: &str);` → `pub fn set_type(this: &UnderlyingByteSource, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn type__js_string(this: &UnderlyingByteSource) -> JsString;`

### `UnderlyingSink`

- `pub fn abort<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>) -> Option<Function<fn(JsValue) -> JsOption<Promise<Undefined>>>>;` → `pub fn abort<W>(this: &UnderlyingSink<W>) -> Option<Function<fn(JsValue) -> JsOption<Promise<Undefined>>>>;`
- `pub fn close<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>) -> Option<Function<fn() -> JsOption<Promise<Undefined>>>>;` → `pub fn close<W>(this: &UnderlyingSink<W>) -> Option<Function<fn() -> JsOption<Promise<Undefined>>>>;`
- `pub fn set_abort<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>, val: &Function<fn(JsValue) -> JsOption<Promise<Undefined>>>);` → `pub fn set_abort<W>(this: &UnderlyingSink<W>, val: &Function<fn(JsValue) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_close<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>, val: &Function<fn() -> JsOption<Promise<Undefined>>>);` → `pub fn set_close<W>(this: &UnderlyingSink<W>, val: &Function<fn() -> JsOption<Promise<Undefined>>>);`
- `pub fn set_start<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>, val: &Function<fn(WritableStreamDefaultController) -> JsOption<Promise<Undefined>>>);` → `pub fn set_start<W>(this: &UnderlyingSink<W>, val: &Function<fn(WritableStreamDefaultController) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_type<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>, val: &str);` → `pub fn set_type<W>(this: &UnderlyingSink<W>, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_write<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>, val: &Function<fn(W, WritableStreamDefaultController) -> JsOption<Promise<Undefined>>>);` → `pub fn set_write<W>(this: &UnderlyingSink<W>, val: &Function<fn(W, WritableStreamDefaultController) -> JsOption<Promise<Undefined>>>);`
- `pub fn start<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>) -> Option<Function<fn(WritableStreamDefaultController) -> JsOption<Promise<Undefined>>>>;` → `pub fn start<W>(this: &UnderlyingSink<W>) -> Option<Function<fn(WritableStreamDefaultController) -> JsOption<Promise<Undefined>>>>;`
- `pub fn type_<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>) -> Option<String>;` → `pub fn type_<W>(this: &UnderlyingSink<W>) -> Option<String>;`
- added → `pub fn type__js_string<W>(this: &UnderlyingSink<W>) -> Option<JsString>;`
- `pub fn write<W: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSink<W>) -> Option<Function<fn(W, WritableStreamDefaultController) -> JsOption<Promise<Undefined>>>>;` → `pub fn write<W>(this: &UnderlyingSink<W>) -> Option<Function<fn(W, WritableStreamDefaultController) -> JsOption<Promise<Undefined>>>>;`
- `impl<W: ::wasm_bindgen::JsGeneric> UnderlyingSink<W>` → `impl<W: ::wasm_bindgen::convert::IntoWasmAbi> UnderlyingSink<W>`
- `pub type UnderlyingSink<W: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type UnderlyingSink<W = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `UnderlyingSinkBuilder`

- `pub fn type_(self, val: &str) -> Self;` → `pub fn type_(self, val: impl ::wasm_bindgen::JsStringLike) -> Self;`
- `impl<W: ::wasm_bindgen::JsGeneric> From<UnderlyingSink<W>> for UnderlyingSinkBuilder<W>` → `impl<W: ::wasm_bindgen::convert::IntoWasmAbi> From<UnderlyingSink<W>> for UnderlyingSinkBuilder<W>`
- `impl<W: ::wasm_bindgen::JsGeneric> UnderlyingSinkBuilder<W>` → `impl<W: ::wasm_bindgen::convert::IntoWasmAbi> UnderlyingSinkBuilder<W>`
- `pub struct UnderlyingSinkBuilder<W: ::wasm_bindgen::JsGeneric> { inner: UnderlyingSink<W> }` → `pub struct UnderlyingSinkBuilder<W> { inner: UnderlyingSink<W> }`

### `UnderlyingSource`

- `pub fn cancel<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>) -> Option<Function<fn(JsValue) -> JsOption<Promise<Undefined>>>>;` → `pub fn cancel<R>(this: &UnderlyingSource<R>) -> Option<Function<fn(JsValue) -> JsOption<Promise<Undefined>>>>;`
- `pub fn expected_length<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>) -> Option<BytesWrittenKind>;` → `pub fn expected_length<R>(this: &UnderlyingSource<R>) -> Option<BytesWrittenKind>;`
- `pub fn pull<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>) -> Option<Function<fn(ReadableStreamDefaultController<R>) -> JsOption<Promise<Undefined>>>>;` → `pub fn pull<R>(this: &UnderlyingSource<R>) -> Option<Function<fn(ReadableStreamDefaultController<R>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn set_cancel<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>, val: &Function<fn(JsValue) -> JsOption<Promise<Undefined>>>);` → `pub fn set_cancel<R>(this: &UnderlyingSource<R>, val: &Function<fn(JsValue) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_expected_length<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>, val: f64);` → `pub fn set_expected_length<R>(this: &UnderlyingSource<R>, val: impl ::js_sys::JsBigIntOrNumberLike);`
- `pub fn set_expected_length_with_big_int<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>, val: &BigInt);` → removed
- `pub fn set_pull<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>, val: &Function<fn(ReadableStreamDefaultController<R>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_pull<R>(this: &UnderlyingSource<R>, val: &Function<fn(ReadableStreamDefaultController<R>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_start<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>, val: &Function<fn(ReadableStreamDefaultController<R>) -> JsOption<Promise<Undefined>>>);` → `pub fn set_start<R>(this: &UnderlyingSource<R>, val: &Function<fn(ReadableStreamDefaultController<R>) -> JsOption<Promise<Undefined>>>);`
- `pub fn set_type<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>, val: Option<&str>);` → `pub fn set_type<R>(this: &UnderlyingSource<R>, val: Option<impl ::wasm_bindgen::JsStringLike>);`
- `pub fn start<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>) -> Option<Function<fn(ReadableStreamDefaultController<R>) -> JsOption<Promise<Undefined>>>>;` → `pub fn start<R>(this: &UnderlyingSource<R>) -> Option<Function<fn(ReadableStreamDefaultController<R>) -> JsOption<Promise<Undefined>>>>;`
- `pub fn type_<R: ::wasm_bindgen::JsGeneric>(this: &UnderlyingSource<R>) -> Option<String>;` → `pub fn type_<R>(this: &UnderlyingSource<R>) -> Option<String>;`
- added → `pub fn type__js_string<R>(this: &UnderlyingSource<R>) -> Option<JsString>;`
- `impl<R: ::wasm_bindgen::JsGeneric> UnderlyingSource<R>` → `impl<R: ::wasm_bindgen::convert::IntoWasmAbi> UnderlyingSource<R>`
- `pub type UnderlyingSource<R: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type UnderlyingSource<R = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `UnderlyingSourceBuilder`

- `pub fn expected_length(self, val: f64) -> Self;` → `pub fn expected_length(self, val: impl ::js_sys::JsBigIntOrNumberLike) -> Self;`
- `pub fn expected_length_with_big_int(self, val: &BigInt) -> Self;` → removed
- `pub fn type_(self, val: Option<&str>) -> Self;` → `pub fn type_(self, val: Option<impl ::wasm_bindgen::JsStringLike>) -> Self;`
- `impl<R: ::wasm_bindgen::JsGeneric> From<UnderlyingSource<R>> for UnderlyingSourceBuilder<R>` → `impl<R: ::wasm_bindgen::convert::IntoWasmAbi> From<UnderlyingSource<R>> for UnderlyingSourceBuilder<R>`
- `impl<R: ::wasm_bindgen::JsGeneric> UnderlyingSourceBuilder<R>` → `impl<R: ::wasm_bindgen::convert::IntoWasmAbi> UnderlyingSourceBuilder<R>`
- `pub struct UnderlyingSourceBuilder<R: ::wasm_bindgen::JsGeneric> { inner: UnderlyingSource<R> }` → `pub struct UnderlyingSourceBuilder<R> { inner: UnderlyingSource<R> }`

### `ValueKind`

- `VecOfBool (Vec<Boolean>)` → `VecOfBool (Vec<bool>)`

### `WebSocket`

- `pub fn close_with_code_and_reason(this: &WebSocket, code: u16, reason: &str) -> Result<(), crate::Error>;` → `pub fn close_with_code_and_reason(this: &WebSocket, code: u16, reason: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- added → `pub fn extensions_js_string(this: &WebSocket) -> Option<JsString>;`
- `pub fn new(url: &str) -> Result<WebSocket, crate::Error>;` → `pub fn new(url: impl ::wasm_bindgen::JsStringLike) -> Result<WebSocket, crate::Error>;`
- `pub fn new_with_slice(url: &str, protocols: &[String]) -> Result<WebSocket, crate::Error>;` → `pub fn new_with_slice(url: impl ::wasm_bindgen::JsStringLike, protocols: &[impl ::wasm_bindgen::JsStringLike]) -> Result<WebSocket, crate::Error>;`
- `pub fn new_with_str(url: &str, protocols: &str) -> Result<WebSocket, crate::Error>;` → `pub fn new_with_str(url: impl ::wasm_bindgen::JsStringLike, protocols: impl ::wasm_bindgen::JsStringLike) -> Result<WebSocket, crate::Error>;`
- added → `pub fn protocol_js_string(this: &WebSocket) -> Option<JsString>;`
- `pub fn send_with_str(this: &WebSocket, message: &str) -> Result<(), crate::Error>;` → `pub fn send_with_str(this: &WebSocket, message: impl ::wasm_bindgen::JsStringLike) -> Result<(), crate::Error>;`
- added → `pub fn url_js_string(this: &WebSocket) -> Option<JsString>;`

### `WebSocketRequestResponsePair`

- `pub fn new(request: &str, response: &str) -> Result<WebSocketRequestResponsePair, crate::Error>;` → `pub fn new(request: impl ::wasm_bindgen::JsStringLike, response: impl ::wasm_bindgen::JsStringLike) -> Result<WebSocketRequestResponsePair, crate::Error>;`
- added → `pub fn request_js_string(this: &WebSocketRequestResponsePair) -> JsString;`
- added → `pub fn response_js_string(this: &WebSocketRequestResponsePair) -> JsString;`

### `WorkerVersionMetadata`

- added → `pub fn id_js_string(this: &WorkerVersionMetadata) -> JsString;`
- `pub fn new(id: &str, tag: &str, timestamp: &str) -> WorkerVersionMetadata;` → `pub fn new(id: impl ::wasm_bindgen::JsStringLike, tag: impl ::wasm_bindgen::JsStringLike, timestamp: impl ::wasm_bindgen::JsStringLike) -> WorkerVersionMetadata;`
- `pub fn set_id(this: &WorkerVersionMetadata, val: &str);` → `pub fn set_id(this: &WorkerVersionMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_tag(this: &WorkerVersionMetadata, val: &str);` → `pub fn set_tag(this: &WorkerVersionMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- `pub fn set_timestamp(this: &WorkerVersionMetadata, val: &str);` → `pub fn set_timestamp(this: &WorkerVersionMetadata, val: impl ::wasm_bindgen::JsStringLike);`
- added → `pub fn tag_js_string(this: &WorkerVersionMetadata) -> JsString;`
- added → `pub fn timestamp_js_string(this: &WorkerVersionMetadata) -> JsString;`

### `WritableStream`

- `pub async fn abort<W: ::wasm_bindgen::JsGeneric>(this: &WritableStream<W>) -> Result<(), crate::Error>;` → `pub async fn abort<W>(this: &WritableStream<W>) -> Result<(), crate::Error>;`
- `pub fn abort_promise<W: ::wasm_bindgen::JsGeneric>(this: &WritableStream<W>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn abort_promise<W>(this: &WritableStream<W>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn abort_with_reason<W: ::wasm_bindgen::JsGeneric>(this: &WritableStream<W>, reason: &JsValue) -> Result<(), crate::Error>;` → `pub async fn abort_with_reason<W>(this: &WritableStream<W>, reason: &JsValue) -> Result<(), crate::Error>;`
- `pub fn abort_with_reason_promise<W: ::wasm_bindgen::JsGeneric>(this: &WritableStream<W>, reason: &JsValue) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn abort_with_reason_promise<W>(this: &WritableStream<W>, reason: &JsValue) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn close<W: ::wasm_bindgen::JsGeneric>(this: &WritableStream<W>) -> Result<(), crate::Error>;` → `pub async fn close<W>(this: &WritableStream<W>) -> Result<(), crate::Error>;`
- `pub fn close_promise<W: ::wasm_bindgen::JsGeneric>(this: &WritableStream<W>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn close_promise<W>(this: &WritableStream<W>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn get_writer<W: ::wasm_bindgen::JsGeneric>(this: &WritableStream<W>) -> Result<WritableStreamDefaultWriter<W>, crate::Error>;` → `pub fn get_writer<W>(this: &WritableStream<W>) -> Result<WritableStreamDefaultWriter<W>, crate::Error>;`
- `pub fn locked<W: ::wasm_bindgen::JsGeneric>(this: &WritableStream<W>) -> bool;` → `pub fn locked<W>(this: &WritableStream<W>) -> bool;`
- `pub fn new() -> Result<WritableStream, crate::Error>;` → `pub fn new<W>() -> Result<WritableStream<W>, crate::Error>;`
- `pub fn new_with_underlying_sink(underlying_sink: &UnderlyingSink) -> Result<WritableStream, crate::Error>;` → `pub fn new_with_underlying_sink<W>(underlying_sink: &UnderlyingSink) -> Result<WritableStream<W>, crate::Error>;`
- `pub fn new_with_underlying_sink_and_queuing_strategy(underlying_sink: &UnderlyingSink, queuing_strategy: &QueuingStrategy) -> Result<WritableStream, crate::Error>;` → `pub fn new_with_underlying_sink_and_queuing_strategy<W>(underlying_sink: &UnderlyingSink, queuing_strategy: &QueuingStrategy) -> Result<WritableStream<W>, crate::Error>;`
- `pub type WritableStream<W: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type WritableStream<W = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `WritableStreamDefaultWriter`

- `pub async fn abort<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Result<(), crate::Error>;` → `pub async fn abort<W>(this: &WritableStreamDefaultWriter<W>) -> Result<(), crate::Error>;`
- `pub fn abort_promise<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn abort_promise<W>(this: &WritableStreamDefaultWriter<W>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn abort_with_reason<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>, reason: &JsValue) -> Result<(), crate::Error>;` → `pub async fn abort_with_reason<W>(this: &WritableStreamDefaultWriter<W>, reason: &JsValue) -> Result<(), crate::Error>;`
- `pub fn abort_with_reason_promise<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>, reason: &JsValue) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn abort_with_reason_promise<W>(this: &WritableStreamDefaultWriter<W>, reason: &JsValue) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn close<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Result<(), crate::Error>;` → `pub async fn close<W>(this: &WritableStreamDefaultWriter<W>) -> Result<(), crate::Error>;`
- `pub fn close_promise<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn close_promise<W>(this: &WritableStreamDefaultWriter<W>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub fn closed<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Promise<Undefined>;` → `pub fn closed<W>(this: &WritableStreamDefaultWriter<W>) -> Promise<Undefined>;`
- `pub fn desired_size<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Option<f64>;` → `pub fn desired_size<W>(this: &WritableStreamDefaultWriter<W>) -> Option<f64>;`
- `pub fn new(stream: &WritableStream) -> Result<WritableStreamDefaultWriter, crate::Error>;` → `pub fn new<W>(stream: &WritableStream) -> Result<WritableStreamDefaultWriter<W>, crate::Error>;`
- `pub fn ready<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Promise<Undefined>;` → `pub fn ready<W>(this: &WritableStreamDefaultWriter<W>) -> Promise<Undefined>;`
- `pub fn release_lock<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Result<(), crate::Error>;` → `pub fn release_lock<W>(this: &WritableStreamDefaultWriter<W>) -> Result<(), crate::Error>;`
- `pub async fn write<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Result<(), crate::Error>;` → `pub async fn write<W>(this: &WritableStreamDefaultWriter<W>) -> Result<(), crate::Error>;`
- `pub fn write_promise<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn write_promise<W>(this: &WritableStreamDefaultWriter<W>) -> Result<Promise<Undefined>, crate::Error>;`
- `pub async fn write_with_chunk<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>, chunk: &W) -> Result<(), crate::Error>;` → `pub async fn write_with_chunk<W>(this: &WritableStreamDefaultWriter<W>, chunk: W) -> Result<(), crate::Error>;`
- `pub fn write_with_chunk_promise<W: ::wasm_bindgen::JsGeneric>(this: &WritableStreamDefaultWriter<W>, chunk: &W) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn write_with_chunk_promise<W>(this: &WritableStreamDefaultWriter<W>, chunk: W) -> Result<Promise<Undefined>, crate::Error>;`
- `pub type WritableStreamDefaultWriter<W: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type WritableStreamDefaultWriter<W = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `cloudflare::email::EmailMessage`

- added → `pub fn from_js_string(this: &EmailMessage) -> JsString;`
- `pub fn new(from: &str, to: &str, raw: &super::super::ReadableStream) -> Result<EmailMessage, crate::Error>;` → `pub fn new(from: impl ::wasm_bindgen::JsStringLike, to: impl ::wasm_bindgen::JsStringLike, raw: &super::super::ReadableStream) -> Result<EmailMessage, crate::Error>;`
- `pub fn new_with_str(from: &str, to: &str, raw: &str) -> Result<EmailMessage, crate::Error>;` → `pub fn new_with_str(from: impl ::wasm_bindgen::JsStringLike, to: impl ::wasm_bindgen::JsStringLike, raw: impl ::wasm_bindgen::JsStringLike) -> Result<EmailMessage, crate::Error>;`
- added → `pub fn to_js_string(this: &EmailMessage) -> JsString;`

### `cloudflare::sockets::(free functions)`

- `pub fn connect(address: &str) -> Result<super::super::Socket, crate::Error>;` → `pub fn connect(address: impl ::wasm_bindgen::JsStringLike) -> Result<super::super::Socket, crate::Error>;`
- `pub fn connect_with_str_and_options(address: &str, options: &super::super::SocketOptions) -> Result<super::super::Socket, crate::Error>;` → `pub fn connect_with_str_and_options(address: impl ::wasm_bindgen::JsStringLike, options: &super::super::SocketOptions) -> Result<super::super::Socket, crate::Error>;`

### `tail_stream::AlarmEventInfo`

- added → `pub fn type__js_string(this: &AlarmEventInfo) -> JsString;`

### `tail_stream::Attribute`

- added → `pub fn name_js_string(this: &Attribute) -> JsString;`

### `tail_stream::Attributes`

- added → `pub fn type__js_string(this: &Attributes) -> JsString;`

### `tail_stream::ConnectEventInfo`

- added → `pub fn type__js_string(this: &ConnectEventInfo) -> JsString;`

### `tail_stream::CustomEventInfo`

- added → `pub fn type__js_string(this: &CustomEventInfo) -> JsString;`

### `tail_stream::DiagnosticChannelEvent`

- added → `pub fn channel_js_string(this: &DiagnosticChannelEvent) -> JsString;`
- added → `pub fn type__js_string(this: &DiagnosticChannelEvent) -> JsString;`

### `tail_stream::DroppedEventsDiagnostic`

- added → `pub fn diagnostics_type_js_string(this: &DroppedEventsDiagnostic) -> JsString;`

### `tail_stream::EmailEventInfo`

- added → `pub fn mail_from_js_string(this: &EmailEventInfo) -> JsString;`
- added → `pub fn rcpt_to_js_string(this: &EmailEventInfo) -> JsString;`
- added → `pub fn type__js_string(this: &EmailEventInfo) -> JsString;`

### `tail_stream::Exception`

- added → `pub fn message_js_string(this: &Exception) -> JsString;`
- added → `pub fn name_js_string(this: &Exception) -> JsString;`
- added → `pub fn stack_js_string(this: &Exception) -> Option<JsString>;`
- added → `pub fn type__js_string(this: &Exception) -> JsString;`

### `tail_stream::FetchEventInfo`

- added → `pub fn method_js_string(this: &FetchEventInfo) -> JsString;`
- added → `pub fn type__js_string(this: &FetchEventInfo) -> JsString;`
- added → `pub fn url_js_string(this: &FetchEventInfo) -> JsString;`

### `tail_stream::FetchResponseInfo`

- added → `pub fn type__js_string(this: &FetchResponseInfo) -> JsString;`

### `tail_stream::Header`

- added → `pub fn name_js_string(this: &Header) -> JsString;`
- added → `pub fn value_js_string(this: &Header) -> JsString;`

### `tail_stream::HibernatableWebSocketEventInfo`

- added → `pub fn type__js_string(this: &HibernatableWebSocketEventInfo) -> JsString;`

### `tail_stream::HibernatableWebSocketEventInfoClose`

- added → `pub fn type__js_string(this: &HibernatableWebSocketEventInfoClose) -> JsString;`

### `tail_stream::HibernatableWebSocketEventInfoError`

- added → `pub fn type__js_string(this: &HibernatableWebSocketEventInfoError) -> JsString;`

### `tail_stream::HibernatableWebSocketEventInfoMessage`

- added → `pub fn type__js_string(this: &HibernatableWebSocketEventInfoMessage) -> JsString;`

### `tail_stream::JsRpcEventInfo`

- added → `pub fn type__js_string(this: &JsRpcEventInfo) -> JsString;`

### `tail_stream::Log`

- added → `pub fn type__js_string(this: &Log) -> JsString;`

### `tail_stream::Onset`

- added → `pub fn dispatch_namespace_js_string(this: &Onset) -> Option<JsString>;`
- added → `pub fn durable_object_id_js_string(this: &Onset) -> Option<JsString>;`
- added → `pub fn entrypoint_js_string(this: &Onset) -> Option<JsString>;`
- added → `pub fn execution_model_js_string(this: &Onset) -> JsString;`
- added → `pub fn script_name_js_string(this: &Onset) -> Option<JsString>;`
- added → `pub fn script_tags_js_string(this: &Onset) -> Option<Array<JsString>>;`
- added → `pub fn span_id_js_string(this: &Onset) -> JsString;`
- added → `pub fn type__js_string(this: &Onset) -> JsString;`

### `tail_stream::Outcome`

- added → `pub fn type__js_string(this: &Outcome) -> JsString;`

### `tail_stream::QueueEventInfo`

- added → `pub fn queue_name_js_string(this: &QueueEventInfo) -> JsString;`
- added → `pub fn type__js_string(this: &QueueEventInfo) -> JsString;`

### `tail_stream::Return`

- added → `pub fn type__js_string(this: &Return) -> JsString;`

### `tail_stream::ScheduledEventInfo`

- added → `pub fn cron_js_string(this: &ScheduledEventInfo) -> JsString;`
- added → `pub fn type__js_string(this: &ScheduledEventInfo) -> JsString;`

### `tail_stream::ScriptVersion`

- added → `pub fn id_js_string(this: &ScriptVersion) -> JsString;`
- added → `pub fn message_js_string(this: &ScriptVersion) -> Option<JsString>;`
- added → `pub fn tag_js_string(this: &ScriptVersion) -> Option<JsString>;`

### `tail_stream::SpanClose`

- added → `pub fn type__js_string(this: &SpanClose) -> JsString;`

### `tail_stream::SpanContext`

- added → `pub fn span_id_js_string(this: &SpanContext) -> Option<JsString>;`
- added → `pub fn trace_id_js_string(this: &SpanContext) -> JsString;`

### `tail_stream::SpanOpen`

- added → `pub fn name_js_string(this: &SpanOpen) -> JsString;`
- added → `pub fn span_id_js_string(this: &SpanOpen) -> JsString;`
- added → `pub fn type__js_string(this: &SpanOpen) -> JsString;`

### `tail_stream::StreamDiagnostic`

- added → `pub fn type__js_string(this: &StreamDiagnostic) -> JsString;`

### `tail_stream::TailEvent`

- `pub fn event<Event: ::wasm_bindgen::JsGeneric>(this: &TailEvent<Event>) -> Event;` → `pub fn event<Event>(this: &TailEvent<Event>) -> Event;`
- `pub fn invocation_id<Event: ::wasm_bindgen::JsGeneric>(this: &TailEvent<Event>) -> String;` → `pub fn invocation_id<Event>(this: &TailEvent<Event>) -> String;`
- added → `pub fn invocation_id_js_string<Event>(this: &TailEvent<Event>) -> JsString;`
- `pub fn sequence<Event: ::wasm_bindgen::JsGeneric>(this: &TailEvent<Event>) -> f64;` → `pub fn sequence<Event>(this: &TailEvent<Event>) -> f64;`
- `pub fn span_context<Event: ::wasm_bindgen::JsGeneric>(this: &TailEvent<Event>) -> SpanContext;` → `pub fn span_context<Event>(this: &TailEvent<Event>) -> SpanContext;`
- `pub fn timestamp<Event: ::wasm_bindgen::JsGeneric>(this: &TailEvent<Event>) -> Date;` → `pub fn timestamp<Event>(this: &TailEvent<Event>) -> Date;`
- `impl<Event: ::wasm_bindgen::JsGeneric> TailEvent<Event>` → `impl<Event: ::wasm_bindgen::convert::IntoWasmAbi> TailEvent<Event>`
- `pub type TailEvent<Event: ::wasm_bindgen::JsGeneric = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)` → `pub type TailEvent<Event = JsValue>; extends(Object) derive(Debug, Clone, PartialEq, Eq)`

### `tail_stream::TailStreamErrorInfo`

- added → `pub fn message_js_string(this: &TailStreamErrorInfo) -> JsString;`
- added → `pub fn name_js_string(this: &TailStreamErrorInfo) -> JsString;`
- added → `pub fn stack_js_string(this: &TailStreamErrorInfo) -> Option<JsString>;`

### `tail_stream::TraceEventInfo`

- added → `pub fn type__js_string(this: &TraceEventInfo) -> JsString;`

### `tail_stream::TracePreviewInfo`

- added → `pub fn id_js_string(this: &TracePreviewInfo) -> JsString;`
- added → `pub fn name_js_string(this: &TracePreviewInfo) -> JsString;`
- added → `pub fn slug_js_string(this: &TracePreviewInfo) -> JsString;`

## Part 2: #122's share (mono `45a278c` → mono `01ed819`)

33 owners touched; 118 items changed, 0 removed, 18 added, 5292 unchanged.

### `CachePurgeOptions`

- added → `pub fn path_prefixes_js_string(this: &CachePurgeOptions) -> Option<Array<JsString>>;`
- `pub fn set_path_prefixes(this: &CachePurgeOptions, val: &[String]);` → `pub fn set_path_prefixes(this: &CachePurgeOptions, val: &[impl ::wasm_bindgen::JsStringLike]);`
- `pub fn set_tags(this: &CachePurgeOptions, val: &[String]);` → `pub fn set_tags(this: &CachePurgeOptions, val: &[impl ::wasm_bindgen::JsStringLike]);`
- added → `pub fn tags_js_string(this: &CachePurgeOptions) -> Option<Array<JsString>>;`

### `CachePurgeOptionsBuilder`

- `pub fn path_prefixes(self, val: &[String]) -> Self;` → `pub fn path_prefixes(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`
- `pub fn tags(self, val: &[String]) -> Self;` → `pub fn tags(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `CloudflareAccessIdentity`

- added → `pub fn amr_js_string(this: &CloudflareAccessIdentity) -> Option<Array<JsString>>;`
- `pub fn set_amr(this: &CloudflareAccessIdentity, val: &[String]);` → `pub fn set_amr(this: &CloudflareAccessIdentity, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `CloudflareAccessIdentityBuilder`

- `pub fn amr(self, val: &[String]) -> Self;` → `pub fn amr(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `Console`

- `pub fn table_with_tabular_data_and_properties(this: &Console, tabular_data: &JsValue, properties: &[String]) -> Result<(), crate::Error>;` → `pub fn table_with_tabular_data_and_properties(this: &Console, tabular_data: &JsValue, properties: &[impl ::wasm_bindgen::JsStringLike]) -> Result<(), crate::Error>;`

### `Container`

- `pub async fn exec(this: &Container, cmd: &[String]) -> Result<ExecProcess, crate::Error>;` → `pub async fn exec(this: &Container, cmd: &[impl ::wasm_bindgen::JsStringLike]) -> Result<ExecProcess, crate::Error>;`
- `pub fn exec_promise(this: &Container, cmd: &[String]) -> Result<Promise<ExecProcess>, crate::Error>;` → `pub fn exec_promise(this: &Container, cmd: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<ExecProcess>, crate::Error>;`
- `pub async fn exec_with_options(this: &Container, cmd: &[String], options: &ContainerExecOptions) -> Result<ExecProcess, crate::Error>;` → `pub async fn exec_with_options(this: &Container, cmd: &[impl ::wasm_bindgen::JsStringLike], options: &ContainerExecOptions) -> Result<ExecProcess, crate::Error>;`
- `pub fn exec_with_options_promise(this: &Container, cmd: &[String], options: &ContainerExecOptions) -> Result<Promise<ExecProcess>, crate::Error>;` → `pub fn exec_with_options_promise(this: &Container, cmd: &[impl ::wasm_bindgen::JsStringLike], options: &ContainerExecOptions) -> Result<Promise<ExecProcess>, crate::Error>;`

### `ContainerStartupOptions`

- added → `pub fn entrypoint_js_string(this: &ContainerStartupOptions) -> Option<Array<JsString>>;`
- `pub fn set_entrypoint(this: &ContainerStartupOptions, val: &[String]);` → `pub fn set_entrypoint(this: &ContainerStartupOptions, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `ContainerStartupOptionsBuilder`

- `pub fn entrypoint(self, val: &[String]) -> Self;` → `pub fn entrypoint(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `CryptoKey`

- added → `pub fn usages_js_string(this: &CryptoKey) -> Array<JsString>;`

### `DurableObjectState`

- `pub fn accept_web_socket_with_tags<Props>(this: &DurableObjectState<Props>, ws: &WebSocket, tags: &[String]) -> Result<(), crate::Error>;` → `pub fn accept_web_socket_with_tags<Props>(this: &DurableObjectState<Props>, ws: &WebSocket, tags: &[impl ::wasm_bindgen::JsStringLike]) -> Result<(), crate::Error>;`
- added → `pub fn get_tags_js_string<Props>(this: &DurableObjectState<Props>, ws: &WebSocket) -> Result<Array<JsString>, crate::Error>;`

### `DurableObjectStorage`

- `pub async fn delete_with_keys(this: &DurableObjectStorage, keys: &[String]) -> Result<usize, crate::Error>;` → `pub async fn delete_with_keys(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<usize, crate::Error>;`
- `pub async fn delete_with_keys_and_options(this: &DurableObjectStorage, keys: &[String], options: &DurableObjectPutOptions) -> Result<usize, crate::Error>;` → `pub async fn delete_with_keys_and_options(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectPutOptions) -> Result<usize, crate::Error>;`
- `pub fn delete_with_keys_and_options_promise(this: &DurableObjectStorage, keys: &[String], options: &DurableObjectPutOptions) -> Result<Promise<usize>, crate::Error>;` → `pub fn delete_with_keys_and_options_promise(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectPutOptions) -> Result<Promise<usize>, crate::Error>;`
- `pub fn delete_with_keys_promise(this: &DurableObjectStorage, keys: &[String]) -> Result<Promise<usize>, crate::Error>;` → `pub fn delete_with_keys_promise(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<usize>, crate::Error>;`
- `pub async fn get_with_keys(this: &DurableObjectStorage, keys: &[String]) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_keys(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub async fn get_with_keys_and_options(this: &DurableObjectStorage, keys: &[String], options: &DurableObjectGetOptions) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_keys_and_options(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectGetOptions) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub fn get_with_keys_and_options_promise(this: &DurableObjectStorage, keys: &[String], options: &DurableObjectGetOptions) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_keys_and_options_promise(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectGetOptions) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `pub fn get_with_keys_promise(this: &DurableObjectStorage, keys: &[String]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_keys_promise(this: &DurableObjectStorage, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`

### `DurableObjectTransaction`

- `pub async fn delete_with_keys(this: &DurableObjectTransaction, keys: &[String]) -> Result<usize, crate::Error>;` → `pub async fn delete_with_keys(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<usize, crate::Error>;`
- `pub async fn delete_with_keys_and_options(this: &DurableObjectTransaction, keys: &[String], options: &DurableObjectPutOptions) -> Result<usize, crate::Error>;` → `pub async fn delete_with_keys_and_options(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectPutOptions) -> Result<usize, crate::Error>;`
- `pub fn delete_with_keys_and_options_promise(this: &DurableObjectTransaction, keys: &[String], options: &DurableObjectPutOptions) -> Result<Promise<usize>, crate::Error>;` → `pub fn delete_with_keys_and_options_promise(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectPutOptions) -> Result<Promise<usize>, crate::Error>;`
- `pub fn delete_with_keys_promise(this: &DurableObjectTransaction, keys: &[String]) -> Result<Promise<usize>, crate::Error>;` → `pub fn delete_with_keys_promise(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<usize>, crate::Error>;`
- `pub async fn get_with_keys(this: &DurableObjectTransaction, keys: &[String]) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_keys(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub async fn get_with_keys_and_options(this: &DurableObjectTransaction, keys: &[String], options: &DurableObjectGetOptions) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_with_keys_and_options(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectGetOptions) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub fn get_with_keys_and_options_promise(this: &DurableObjectTransaction, keys: &[String], options: &DurableObjectGetOptions) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_keys_and_options_promise(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike], options: &DurableObjectGetOptions) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `pub fn get_with_keys_promise(this: &DurableObjectTransaction, keys: &[String]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_with_keys_promise(this: &DurableObjectTransaction, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`

### `Headers`

- added → `pub fn get_all_js_string(this: &Headers, name: impl ::wasm_bindgen::JsStringLike) -> Result<Array<JsString>, JsException>;`
- added → `pub fn get_set_cookie_js_string(this: &Headers) -> Result<Array<JsString>, crate::Error>;`

### `JsonWebKey`

- added → `pub fn key_ops_js_string(this: &JsonWebKey) -> Option<Array<JsString>>;`
- `pub fn set_key_ops(this: &JsonWebKey, val: &[String]);` → `pub fn set_key_ops(this: &JsonWebKey, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `JsonWebKeyBuilder`

- `pub fn key_ops(self, val: &[String]) -> Self;` → `pub fn key_ops(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `KVNamespace`

- `pub async fn get_json_with_slice(&self, key: &[String]) -> Result<Map<JsString, JsValue>, crate::Error>;` → `pub async fn get_json_with_slice(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `async fn get_json_with_slice_js(this: &KVNamespace, key: &[String], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, JsValue>, crate::Error>;` → `async fn get_json_with_slice_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, JsValue>, crate::Error>;`
- `pub fn get_json_with_slice_promise(&self, key: &[String]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `pub fn get_json_with_slice_promise(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `fn get_json_with_slice_promise_js(this: &KVNamespace, key: &[String], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;` → `fn get_json_with_slice_promise_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, JsValue>>, crate::Error>;`
- `pub async fn get_text_with_slice(&self, key: &[String]) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `pub async fn get_text_with_slice(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `async fn get_text_with_slice_js(this: &KVNamespace, key: &[String], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `async fn get_text_with_slice_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `pub fn get_text_with_slice_promise(&self, key: &[String]) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `pub fn get_text_with_slice_promise(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`
- `fn get_text_with_slice_promise_js(this: &KVNamespace, key: &[String], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `fn get_text_with_slice_promise_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`
- `pub async fn get_with_key_slice_and_options_kv_namespace_get_options(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<String>) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `pub async fn get_with_key_slice_and_options_kv_namespace_get_options(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<String>) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `pub fn get_with_key_slice_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<String>) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `pub fn get_with_key_slice_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<String>) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`
- `pub async fn get_with_metadata_json_with_slice(&self, key: &[String]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_json_with_slice(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;`
- `async fn get_with_metadata_json_with_slice_js(this: &KVNamespace, key: &[String], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;` → `async fn get_with_metadata_json_with_slice_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>, crate::Error>;`
- `pub fn get_with_metadata_json_with_slice_promise(&self, key: &[String]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_json_with_slice_promise(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>>, crate::Error>;`
- `fn get_with_metadata_json_with_slice_promise_js(this: &KVNamespace, key: &[String], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>>, crate::Error>;` → `fn get_with_metadata_json_with_slice_promise_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<JsValue, JsValue>>>, crate::Error>;`
- `pub async fn get_with_metadata_text_with_slice(&self, key: &[String]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_text_with_slice(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `async fn get_with_metadata_text_with_slice_js(this: &KVNamespace, key: &[String], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;` → `async fn get_with_metadata_text_with_slice_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub fn get_with_metadata_text_with_slice_promise(&self, key: &[String]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_text_with_slice_promise(&self, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `fn get_with_metadata_text_with_slice_promise_js(this: &KVNamespace, key: &[String], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;` → `fn get_with_metadata_text_with_slice_promise_js(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], r#type: impl ::wasm_bindgen::JsStringLike) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `pub async fn get_with_metadata_with_key_slice_and_options_kv_namespace_get_options(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<String>) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_with_key_slice_and_options_kv_namespace_get_options(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<String>) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub fn get_with_metadata_with_key_slice_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<String>) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_with_key_slice_and_options_kv_namespace_get_options_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<String>) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `pub async fn get_with_metadata_with_slice(this: &KVNamespace, key: &[String]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_with_slice(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub async fn get_with_metadata_with_slice_and_kv_namespace_get_options(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<Undefined>) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;` → `pub async fn get_with_metadata_with_slice_and_kv_namespace_get_options(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<Undefined>) -> Result<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>, crate::Error>;`
- `pub fn get_with_metadata_with_slice_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_with_slice_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `pub fn get_with_metadata_with_slice_promise(this: &KVNamespace, key: &[String]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;` → `pub fn get_with_metadata_with_slice_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, KVNamespaceGetWithMetadataResult<String, JsValue>>>, crate::Error>;`
- `pub async fn get_with_slice(this: &KVNamespace, key: &[String]) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `pub async fn get_with_slice(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `pub async fn get_with_slice_and_kv_namespace_get_options(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<Undefined>) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;` → `pub async fn get_with_slice_and_kv_namespace_get_options(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<Undefined>) -> Result<Map<JsString, JsNullable<JsString>>, crate::Error>;`
- `pub fn get_with_slice_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &[String], options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `pub fn get_with_slice_and_kv_namespace_get_options_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike], options: &KVNamespaceGetOptions<Undefined>) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`
- `pub fn get_with_slice_promise(this: &KVNamespace, key: &[String]) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;` → `pub fn get_with_slice_promise(this: &KVNamespace, key: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Map<JsString, JsNullable<JsString>>>, crate::Error>;`

### `R2Bucket`

- `pub async fn delete_with_slice(this: &R2Bucket, keys: &[String]) -> Result<(), crate::Error>;` → `pub async fn delete_with_slice(this: &R2Bucket, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<(), crate::Error>;`
- `pub fn delete_with_slice_promise(this: &R2Bucket, keys: &[String]) -> Result<Promise<Undefined>, crate::Error>;` → `pub fn delete_with_slice_promise(this: &R2Bucket, keys: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<Undefined>, crate::Error>;`

### `R2ListOptions`

- added → `pub fn include_js_string(this: &R2ListOptions) -> Option<Array<JsString>>;`
- `pub fn set_include(this: &R2ListOptions, val: &[String]);` → `pub fn set_include(this: &R2ListOptions, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `R2ListOptionsBuilder`

- `pub fn include(self, val: &[String]) -> Self;` → `pub fn include(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `R2Objects`

- `pub fn builder_false(objects: &[R2Object], delimited_prefixes: &[String]) -> R2ObjectsBuilder;` → `pub fn builder_false(objects: &[R2Object], delimited_prefixes: &[impl ::wasm_bindgen::JsStringLike]) -> R2ObjectsBuilder;`
- `pub fn builder_true(objects: &[R2Object], delimited_prefixes: &[String]) -> R2ObjectsBuilder;` → `pub fn builder_true(objects: &[R2Object], delimited_prefixes: &[impl ::wasm_bindgen::JsStringLike]) -> R2ObjectsBuilder;`
- added → `pub fn delimited_prefixes_js_string(this: &R2Objects) -> Array<JsString>;`
- `pub fn new_false(objects: &[R2Object], delimited_prefixes: &[String]) -> R2Objects;` → `pub fn new_false(objects: &[R2Object], delimited_prefixes: &[impl ::wasm_bindgen::JsStringLike]) -> R2Objects;`
- `pub fn new_true(objects: &[R2Object], delimited_prefixes: &[String]) -> R2Objects;` → `pub fn new_true(objects: &[R2Object], delimited_prefixes: &[impl ::wasm_bindgen::JsStringLike]) -> R2Objects;`
- `pub fn set_delimited_prefixes(this: &R2Objects, val: &[String]);` → `pub fn set_delimited_prefixes(this: &R2Objects, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `RequestInitCfProperties`

- added → `pub fn cache_tags_js_string(this: &RequestInitCfProperties) -> Option<Array<JsString>>;`
- `pub fn set_cache_tags(this: &RequestInitCfProperties, val: &[String]);` → `pub fn set_cache_tags(this: &RequestInitCfProperties, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `RequestInitCfPropertiesBuilder`

- `pub fn cache_tags(self, val: &[String]) -> Self;` → `pub fn cache_tags(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `RequestInitCfPropertiesVaryAcceptHeader`

- added → `pub fn media_types_js_string(this: &RequestInitCfPropertiesVaryAcceptHeader) -> Option<Array<JsString>>;`
- `pub fn set_media_types(this: &RequestInitCfPropertiesVaryAcceptHeader, val: &[String]);` → `pub fn set_media_types(this: &RequestInitCfPropertiesVaryAcceptHeader, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `RequestInitCfPropertiesVaryAcceptHeaderBuilder`

- `pub fn media_types(self, val: &[String]) -> Self;` → `pub fn media_types(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `RequestInitCfPropertiesVaryAcceptLanguageHeader`

- added → `pub fn languages_js_string(this: &RequestInitCfPropertiesVaryAcceptLanguageHeader) -> Option<Array<JsString>>;`
- `pub fn set_languages(this: &RequestInitCfPropertiesVaryAcceptLanguageHeader, val: &[String]);` → `pub fn set_languages(this: &RequestInitCfPropertiesVaryAcceptLanguageHeader, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `RequestInitCfPropertiesVaryAcceptLanguageHeaderBuilder`

- `pub fn languages(self, val: &[String]) -> Self;` → `pub fn languages(self, val: &[impl ::wasm_bindgen::JsStringLike]) -> Self;`

### `SqlStorageCursor`

- added → `pub fn column_names_js_string<T>(this: &SqlStorageCursor<T>) -> Array<JsString>;`
- `pub fn set_column_names<T>(this: &SqlStorageCursor<T>, val: &[String]);` → `pub fn set_column_names<T>(this: &SqlStorageCursor<T>, val: &[impl ::wasm_bindgen::JsStringLike]);`

### `SubtleCrypto`

- `pub async fn derive_key(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn derive_key(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn derive_key_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn derive_key_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn derive_key_with_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn derive_key_with_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn derive_key_with_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn derive_key_with_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn derive_key_with_subtle_crypto_derive_key_algorithm_and_str(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn derive_key_with_subtle_crypto_derive_key_algorithm_and_str(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn derive_key_with_subtle_crypto_derive_key_algorithm_and_str_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn derive_key_with_subtle_crypto_derive_key_algorithm_and_str_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn derive_key_with_subtle_crypto_derive_key_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn derive_key_with_subtle_crypto_derive_key_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn derive_key_with_subtle_crypto_derive_key_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn derive_key_with_subtle_crypto_derive_key_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoDeriveKeyAlgorithm, base_key: &CryptoKey, derived_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn generate_key(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<GenerateKeyReturnKind, crate::Error>;` → `pub async fn generate_key(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<GenerateKeyReturnKind, crate::Error>;`
- `pub fn generate_key_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise, crate::Error>;` → `pub fn generate_key_promise(this: &SubtleCrypto, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise, crate::Error>;`
- `pub async fn generate_key_with_subtle_crypto_generate_key_algorithm(this: &SubtleCrypto, algorithm: &SubtleCryptoGenerateKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<GenerateKeyReturnKind, crate::Error>;` → `pub async fn generate_key_with_subtle_crypto_generate_key_algorithm(this: &SubtleCrypto, algorithm: &SubtleCryptoGenerateKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<GenerateKeyReturnKind, crate::Error>;`
- `pub fn generate_key_with_subtle_crypto_generate_key_algorithm_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoGenerateKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise, crate::Error>;` → `pub fn generate_key_with_subtle_crypto_generate_key_algorithm_promise(this: &SubtleCrypto, algorithm: &SubtleCryptoGenerateKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise, crate::Error>;`
- `pub async fn import_key(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_array_buffer_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_array_buffer_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_array_buffer_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_array_buffer_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &ArrayBuffer, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_json_web_key_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_json_web_key_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_json_web_key_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_json_web_key_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_json_web_key_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_json_web_key_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_json_web_key_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_json_web_key_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &JsonWebKey, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_typed_array_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_typed_array_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_typed_array_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_typed_array_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn import_key_with_typed_array_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn import_key_with_typed_array_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn import_key_with_typed_array_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn import_key_with_typed_array_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, key_data: &Object, algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_array_buffer_and_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_array_buffer_and_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_array_buffer_and_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_array_buffer_and_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_array_buffer_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &ArrayBuffer, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_typed_array_and_str_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_typed_array_and_str_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_typed_array_and_str_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_typed_array_and_str_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_typed_array_and_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_typed_array_and_str_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_typed_array_and_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_typed_array_and_str_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: impl ::wasm_bindgen::JsStringLike, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_str(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_str_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: impl ::wasm_bindgen::JsStringLike, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`
- `pub async fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<CryptoKey, crate::Error>;` → `pub async fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<CryptoKey, crate::Error>;`
- `pub fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[String]) -> Result<Promise<CryptoKey>, crate::Error>;` → `pub fn unwrap_key_with_typed_array_and_subtle_crypto_encrypt_algorithm_and_subtle_crypto_import_key_algorithm_promise(this: &SubtleCrypto, format: impl ::wasm_bindgen::JsStringLike, wrapped_key: &Object, unwrapping_key: &CryptoKey, unwrap_algorithm: &SubtleCryptoEncryptAlgorithm, unwrapped_key_algorithm: &SubtleCryptoImportKeyAlgorithm, extractable: bool, key_usages: &[impl ::wasm_bindgen::JsStringLike]) -> Result<Promise<CryptoKey>, crate::Error>;`

### `TraceItem`

- added → `pub fn script_tags_js_string(this: &TraceItem) -> Option<Array<JsString>>;`

### `URLSearchParams`

- added → `pub fn get_all_js_string(this: &URLSearchParams, name: impl ::wasm_bindgen::JsStringLike) -> Result<Array<JsString>, crate::Error>;`

### `ValueKind`

- `VecOfBool (Vec<Boolean>)` → `VecOfBool (Vec<bool>)`

### `WebSocket`

- `pub fn new_with_slice(url: impl ::wasm_bindgen::JsStringLike, protocols: &[String]) -> Result<WebSocket, crate::Error>;` → `pub fn new_with_slice(url: impl ::wasm_bindgen::JsStringLike, protocols: &[impl ::wasm_bindgen::JsStringLike]) -> Result<WebSocket, crate::Error>;`

### `tail_stream::Onset`

- added → `pub fn script_tags_js_string(this: &Onset) -> Option<Array<JsString>>;`
