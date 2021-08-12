use js_sys::Date as JsDate;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub struct Date {
    js_date: JsDate,
}

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
    pub fn new(init: DateInit) -> Self {
        let val = match init {
            DateInit::Millis(n) => JsValue::from_f64(n as f64),
            DateInit::String(s) => JsValue::from_str(&s),
        };

        Self {
            js_date: JsDate::new(&val),
        }
    }

    pub fn now() -> Self {
        Self {
            js_date: JsDate::new_0(),
        }
    }

    pub fn as_millis(&self) -> u64 {
        self.js_date.get_time() as u64
    }
}

impl ToString for Date {
    fn to_string(&self) -> String {
        self.js_date.to_string().into()
    }
}
