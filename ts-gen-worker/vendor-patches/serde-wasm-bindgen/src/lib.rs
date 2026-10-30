#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_const_for_fn)]

use js_sys::JsString;
use wasm_bindgen::prelude::*;

mod de;
mod error;
mod ser;

pub use de::Deserializer;
pub use error::Error;
pub use ser::Serializer;

type Result<T> = std::result::Result<T, Error>;

fn static_str_to_js(s: &'static str) -> JsString {
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct PtrHasher {
        addr: usize,
    }

    impl std::hash::Hasher for PtrHasher {
        fn write(&mut self, _bytes: &[u8]) {
            unreachable!();
        }

        fn write_usize(&mut self, addr_or_len: usize) {
            if self.addr == 0 {
                self.addr = addr_or_len;
            }
        }

        fn finish(&self) -> u64 {
            self.addr as _
        }
    }

    type PtrBuildHasher = std::hash::BuildHasherDefault<PtrHasher>;

    thread_local! {
        // Since we're mainly optimising for converting the exact same string literal over and over again,
        // which will always have the same pointer, we can speed things up by indexing by the string's pointer
        // instead of its value.
        static CACHE: RefCell<HashMap<*const str, JsString, PtrBuildHasher>> = Default::default();
    }
    CACHE.with(|cache| {
        cache
            .borrow_mut()
            .entry(s)
            .or_insert_with(|| s.into())
            .clone()
    })
}

/// Custom bindings to avoid using fallible `Reflect` for plain objects.
#[wasm_bindgen]
extern "C" {
    type ObjectExt;

    #[wasm_bindgen(method, indexing_getter)]
    fn get_with_ref_key(this: &ObjectExt, key: &JsString) -> JsValue;

    #[wasm_bindgen(method, indexing_setter)]
    fn set(this: &ObjectExt, key: JsString, value: JsValue);
}

/// Converts [`JsValue`] into a Rust type.
pub fn from_value<T: serde::de::DeserializeOwned>(value: JsValue) -> Result<T> {
    T::deserialize(Deserializer::from(value))
}

/// Converts a Rust value into a [`JsValue`].
pub fn to_value<T: serde::ser::Serialize + ?Sized>(value: &T) -> Result<JsValue> {
    value.serialize(&Serializer::new())
}

/// Serialization and deserialization functions that pass JavaScript objects through unchanged.
///
/// This module is compatible with the `serde(with)` annotation, so for example if you create
/// the struct
///
/// ```rust
/// #[derive(serde::Serialize)]
/// struct MyStruct {
///     int_field: i32,
///     #[serde(with = "serde_wasm_bindgen::preserve")]
///     js_field: js_sys::Int8Array,
/// }
///
/// let s = MyStruct {
///     int_field: 5,
///     js_field: js_sys::Int8Array::new_with_length(1000),
/// };
/// ```
///
/// then `serde_wasm_bindgen::to_value(&s)`
/// will return a JsValue representing an object with two fields (`int_field` and `js_field`), where
/// `js_field` will be an `Int8Array` pointing to the same underlying JavaScript object as `s.js_field` does.
pub mod preserve {
    use serde::{de::Error, Deserialize, Serialize};
    use wasm_bindgen::{
        convert::{FromWasmAbi, IntoWasmAbi},
        JsCast, JsValue,
    };

    // Some arbitrary string that no one will collide with unless they try.
    pub(crate) const PRESERVED_VALUE_MAGIC: &str = "1fc430ca-5b7f-4295-92de-33cf2b145d38";

    struct Magic;

    impl<'de> serde::de::Deserialize<'de> for Magic {
        fn deserialize<D: serde::de::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
            struct Visitor;

            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = Magic;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("serde-wasm-bindgen's magic string")
                }

                fn visit_str<E: Error>(self, s: &str) -> Result<Self::Value, E> {
                    if s == PRESERVED_VALUE_MAGIC {
                        Ok(Magic)
                    } else {
                        Err(E::invalid_value(serde::de::Unexpected::Str(s), &self))
                    }
                }
            }

            de.deserialize_str(Visitor)
        }
    }

    #[derive(Serialize)]
    #[serde(rename = "1fc430ca-5b7f-4295-92de-33cf2b145d38")]
    struct PreservedValueSerWrapper(u32);

    // Intentionally asymmetrical wrapper to ensure that only serde-wasm-bindgen preserves roundtrip.
    #[derive(Deserialize)]
    #[serde(rename = "1fc430ca-5b7f-4295-92de-33cf2b145d38")]
    struct PreservedValueDeWrapper(Magic, u32);

    /// Serialize any `JsCast` value.
    ///
    /// When used with the `Serializer` in `serde_wasm_bindgen`, this serializes the value by
    /// passing it through as a `JsValue`.
    ///
    /// This function is compatible with the `serde(serialize_with)` derive annotation.
    pub fn serialize<S: serde::Serializer, T: JsCast>(val: &T, ser: S) -> Result<S::Ok, S::Error> {
        // It's responsibility of serde-wasm-bindgen's Serializer to clone the value.
        // For all other serializers, using reference instead of cloning here will ensure that we don't
        // create accidental leaks.
        PreservedValueSerWrapper(val.as_ref().into_abi()).serialize(ser)
    }

    /// Deserialize any `JsCast` value.
    ///
    /// When used with the `Derializer` in `serde_wasm_bindgen`, this serializes the value by
    /// passing it through as a `JsValue` and casting it.
    ///
    /// This function is compatible with the `serde(deserialize_with)` derive annotation.
    pub fn deserialize<'de, D: serde::Deserializer<'de>, T: JsCast>(de: D) -> Result<T, D::Error> {
        let wrap = PreservedValueDeWrapper::deserialize(de)?;
        // When used with our deserializer this unsafe is correct, because the
        // deserializer just converted a JsValue into_abi.
        //
        // Other deserializers are unlikely to end up here, thanks
        // to the asymmetry between PreservedValueSerWrapper and
        // PreservedValueDeWrapper. Even if some other deserializer ends up
        // here, this may be incorrect but it shouldn't be UB because JsValues
        // are represented using indices into a JS-side (i.e. bounds-checked)
        // array.
        let val: JsValue = unsafe { FromWasmAbi::from_abi(wrap.1) };
        val.dyn_into().map_err(|e| {
            D::Error::custom(format_args!(
                "incompatible JS value {e:?} for type {}",
                std::any::type_name::<T>()
            ))
        })
    }
}
