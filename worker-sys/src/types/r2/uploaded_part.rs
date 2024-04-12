use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;

#[derive(Debug, Clone)]
pub struct R2UploadedPart {
    pub part_number: u16,
    pub etag: String,
}

impl R2UploadedPart {
    pub fn new(part_number: u16, etag: String) -> Self {
        Self { part_number, etag }
    }

    pub fn as_object(&self) -> Object {
        let obj = Object::new();
        Reflect::set(
            &obj,
            &"partNumber".into(),
            &JsValue::from_f64(self.part_number as f64),
        )
        .unwrap();
        Reflect::set(&obj, &"etag".into(), &JsValue::from_str(&self.etag)).unwrap();

        obj
    }
}

impl From<Object> for R2UploadedPart {
    fn from(obj: Object) -> Self {
        Self {
            part_number: Reflect::get(&obj, &"partNumber".into())
                .unwrap()
                .as_f64()
                .unwrap() as u16,
            etag: Reflect::get(&obj, &"etag".into())
                .unwrap()
                .as_string()
                .unwrap(),
        }
    }
}
