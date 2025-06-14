use std::fmt::Display;

use chrono::offset::TimeZone;
use chrono::Datelike;
use js_sys::Date as JsDate;
use wasm_bindgen::JsValue;

/// The equivalent to a JavaScript `Date` Object.
/// ```no_run
/// # use worker::Date;
/// # use worker::DateInit;
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
/// # use worker::DateInit;
/// # use worker::Date;
/// let t1: Date = DateInit::Millis(1630611511000).into();
/// let t2: Date = DateInit::String("Thu, 02 Sep 2021 19:38:31 GMT".to_string()).into();
/// ```
#[derive(Debug)]
pub enum DateInit {
    Millis(u64),
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
    pub fn now() -> Self {
        Self {
            js_date: JsDate::new_0(),
        }
    }

    /// Convert a Date into its number of milliseconds since the Unix epoch.
    pub fn as_millis(&self) -> u64 {
        self.js_date.get_time() as u64
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.js_date.to_string())
    }
}

#[allow(deprecated)]
impl<T: TimeZone> From<chrono::Date<T>> for Date {
    fn from(d: chrono::Date<T>) -> Self {
        Self {
            js_date: JsDate::new_with_year_month_day(
                d.year() as u32,
                d.month() as i32 - 1,
                d.day() as i32,
            ),
        }
    }
}

impl<T: TimeZone> From<chrono::DateTime<T>> for Date {
    fn from(dt: chrono::DateTime<T>) -> Self {
        DateInit::Millis(dt.timestamp_millis() as u64).into()
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
