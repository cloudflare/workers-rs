use js_sys::{ArrayBuffer, Object, Reflect, Uint8Array};
use wasm_bindgen::JsCast;

#[derive(Debug, Clone)]
pub struct R2Checksums {
    pub md5: Option<Vec<u8>>,
    pub sha1: Option<Vec<u8>>,
    pub sha256: Option<Vec<u8>>,
    pub sha384: Option<Vec<u8>>,
    pub sha512: Option<Vec<u8>>,
}

impl R2Checksums {
    pub fn new() -> Self {
        Self {
            md5: None,
            sha1: None,
            sha256: None,
            sha384: None,
            sha512: None,
        }
    }
}

fn get(obj: &Object, key: &str) -> Option<Vec<u8>> {
    let value = Reflect::get(&obj, &key.into());
    if value.is_err() {
        return None;
    }

    let value = value.unwrap().dyn_into::<ArrayBuffer>();
    if value.is_err() {
        return None;
    }

    let array_buffer: ArrayBuffer = value.unwrap();

    let uint8_array = Uint8Array::new(&array_buffer);
    let mut vec = vec![0; uint8_array.length() as usize];
    uint8_array.copy_to(&mut vec);
    Some(vec)
}

impl From<Object> for R2Checksums {
    fn from(obj: Object) -> Self {
        Self {
            md5: get(&obj, "md5"),
            sha1: get(&obj, "sha1"),
            sha256: get(&obj, "sha256"),
            sha384: get(&obj, "sha384"),
            sha512: get(&obj, "sha512"),
        }
    }
}
