use std::fmt::Display;

use js_sys::Date as JsDate;
use wasm_bindgen::JsValue;

/// The equivalent to a JavaScript `Date` Object.
///
/// Stores signed milliseconds since the Unix epoch, including negative values for dates before it.
/// With the optional `jiff` feature, conversions from `jiff::Timestamp` and
/// `jiff::ZonedDateTime` truncate fractional milliseconds toward zero and do not retain time zone
/// information. Calendar-only dates must first be resolved to an instant using an explicit time zone.
///
/// ```no_run
/// # use worker::{Date, DateInit};
/// let now = Date::now();
/// let millis = now.as_millis();
/// // or use a specific point in time:
/// let t1: Date = DateInit::Millis(1630611511000).into();
/// let t2: Date = DateInit::String("Thu, 02 Sep 2021 19:38:31 GMT".to_string()).into();
/// ```
#[derive(Debug, Clone, Eq)]
pub struct Date {
    js_date: JsDate,
}

impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        self.js_date.as_f64() == other.js_date.as_f64()
    }
}

/// Initialize a `Date` by constructing this enum.
/// ```no_run
/// # use worker::{Date, DateInit};
/// let t1: Date = DateInit::Millis(1630611511000).into();
/// let t2: Date = DateInit::String("Thu, 02 Sep 2021 19:38:31 GMT".to_string()).into();
/// ```
#[derive(Debug)]
pub enum DateInit {
    /// Signed milliseconds since the Unix epoch; negative values represent dates before it.
    Millis(i64),
    String(String),
}

impl From<DateInit> for Date {
    fn from(init: DateInit) -> Self {
        Date::new(init)
    }
}

impl Date {
    /// Create a new Date, which requires being initialized from a known DateInit value.
    pub fn new(init: DateInit) -> Self {
        let val = match init {
            DateInit::Millis(n) => JsValue::from_f64(n as f64),
            DateInit::String(s) => JsValue::from_str(&s),
        };

        Self {
            js_date: JsDate::new(&val),
        }
    }

    /// Get the current time, represented by a Date.
    ///
    /// Uses the JavaScript runtime clock without requiring Jiff or its `js` feature.
    /// On deployed Workers this clock only advances after I/O; it is not a monotonic clock
    /// or a CPU timer. See [Performance and timers](https://developers.cloudflare.com/workers/runtime-apis/performance/).
    pub fn now() -> Self {
        Self {
            js_date: JsDate::new_0(),
        }
    }

    /// Return signed milliseconds since the Unix epoch.
    ///
    /// Dates before the Unix epoch have negative values. JavaScript dates have millisecond precision.
    pub fn as_millis(&self) -> i64 {
        self.js_date.get_time() as i64
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.js_date.to_string())
    }
}

#[cfg(feature = "jiff")]
impl From<jiff::Timestamp> for Date {
    fn from(timestamp: jiff::Timestamp) -> Self {
        DateInit::Millis(timestamp.as_millisecond()).into()
    }
}

#[cfg(feature = "jiff")]
impl From<jiff::ZonedDateTime> for Date {
    fn from(date: jiff::ZonedDateTime) -> Self {
        date.timestamp().into()
    }
}

impl From<Date> for JsDate {
    fn from(val: Date) -> Self {
        val.js_date
    }
}

impl From<JsDate> for Date {
    fn from(js_date: JsDate) -> Self {
        Self { js_date }
    }
}
