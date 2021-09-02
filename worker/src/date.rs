use js_sys::Date as JsDate;
use wasm_bindgen::JsValue;

/// The equivalent to a JavaScript `Date` Object.
/// ```no_run
/// let now = Date::now();
/// let millis = now.as_millis();
/// // or use a specific point in time:
/// let t1: Date = DateInit::Millis(1630611511000).into();
/// let t2: Date = DateInit::String("Thu, 02 Sep 2021 19:38:31 GMT".to_string()).into();
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Date {
    js_date: JsDate,
}

/// Initialize a `Date` by constructing this enum.
/// ```no_run
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

impl ToString for Date {
    fn to_string(&self) -> String {
        self.js_date.to_string().into()
    }
}
