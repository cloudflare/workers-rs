---
"workers-rs": major
---

Replace Chrono and Chrono-TZ with Jiff 1.0. `worker::Date` and `ScheduledTime` now accept `jiff::Timestamp` and `jiff::ZonedDateTime` instead of Chrono date-time values. These conversions preserve the instant at millisecond precision. Convert calendar-only dates to an instant with an explicit time zone before passing them to the SDK.

With the `timezone` feature, `Cf::timezone()` now returns `jiff::tz::TimeZone`; missing and invalid zone names return `Error::TimezoneError`. The feature includes the IANA time zone database for runtime lookups. `Cf::timezone_name()` remains available without this feature.

`DateInit::Millis` and `Date::as_millis()` now use `i64` instead of `u64`, so dates before the Unix epoch retain their negative millisecond values. Update explicitly typed callers accordingly.

The SDK does not enable Jiff's `js` feature. Applications that call Jiff's current-time APIs on `wasm32-unknown-unknown` must enable that feature in their own Jiff dependency. `worker::Date::now()` continues to use JavaScript directly.
