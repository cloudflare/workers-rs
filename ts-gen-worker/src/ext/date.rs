//! `worker::Date` was a wrapper around `js_sys::Date`; here `js_sys::Date` is the type.
//! Inherent impls on a foreign type are impossible, so the `worker` constructors and
//! accessors are a `From<DateInit>` conversion plus the `DateExt` extension trait.

use js_sys::Date;

/// A point in time to construct a `Date` from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateInit {
    Millis(u64),
    String(String),
}

impl From<DateInit> for Date {
    fn from(init: DateInit) -> Self {
        match init {
            DateInit::Millis(n) => Date::new_with_value(n as f64),
            DateInit::String(s) => Date::new_with_value(s),
        }
    }
}

/// `worker::Date`'s API on `js_sys::Date`. `js_sys::Date::new_0()` is `worker`'s
/// `Date::now()`; `js_sys::Date::now()` already returns the epoch milliseconds directly.
pub trait DateExt {
    fn from_init(init: DateInit) -> Self;
    fn current() -> Self;
    /// Milliseconds since the Unix epoch.
    fn as_millis(&self) -> u64;
}

impl DateExt for Date {
    fn from_init(init: DateInit) -> Self {
        init.into()
    }

    fn current() -> Self {
        Date::new_0()
    }

    fn as_millis(&self) -> u64 {
        self.get_time() as u64
    }
}
