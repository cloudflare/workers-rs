---
"workers-rs": major
---

Replace Chrono and Chrono-TZ with optional Jiff 1.0 support. Enable the new, disabled-by-default `jiff` feature to convert `jiff::Timestamp` and `jiff::ZonedDateTime` into `worker::Date` and `ScheduledTime`. These replace Chrono conversions and truncate fractional milliseconds toward zero. Resolve calendar-only dates to an instant using an explicit time zone first.

The opt-in `timezone` feature enables `jiff` and bundles the full IANA database; `jiff` alone does not bundle it. `Cf::timezone()` now returns `jiff::tz::TimeZone`, with missing or invalid names returning `Error::TimezoneError`. `Cf::timezone_name()` remains a raw string available without either feature, including for subset databases or custom handling.

`DateInit::Millis` and `Date::as_millis()` now use `i64` instead of `u64`, so dates before the Unix epoch retain their negative millisecond values. Update explicitly typed callers accordingly.

`ScheduledTime` now accepts `worker::Date` as an absolute alarm timestamp without a date library: pass `Date::from(DateInit::Millis(unix_millis))` to `set_alarm`. Integer `i64` and `Duration` inputs remain relative delays, not Unix timestamps.

For Jiff current-time APIs on `wasm32-unknown-unknown`, enable `features = ["js"]` in the application's own Jiff dependency. `worker::Date::now()` needs neither Jiff nor its `js` feature; `std::time` clocks remain unsupported on this target. On deployed Workers, the JavaScript clock only advances after I/O and is not a monotonic or CPU timer; see [Performance and timers](https://developers.cloudflare.com/workers/runtime-apis/performance/).
