use js_sys::{Array, JsString, Map, Number, Object, Uint8Array};
use serde::ser::{self, Error as _, Serialize};
use wasm_bindgen::convert::RefFromWasmAbi;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::preserve::PRESERVED_VALUE_MAGIC;
use crate::{static_str_to_js, Error, ObjectExt};

type Result<T = JsValue> = super::Result<T>;

/// Wraps other serializers into an enum tagged variant form.
///
/// Results in `{"Variant": ...payload...}` for compatibility with serde-json.
pub struct VariantSerializer<S> {
    variant: &'static str,
    inner: S,
}

impl<S> VariantSerializer<S> {
    const fn new(variant: &'static str, inner: S) -> Self {
        Self { variant, inner }
    }

    fn end(self, inner: impl FnOnce(S) -> Result) -> Result {
        let value = inner(self.inner)?;
        let obj = Object::new();
        obj.unchecked_ref::<ObjectExt>()
            .set(static_str_to_js(self.variant), value);
        Ok(obj.into())
    }
}

impl<S: ser::SerializeTupleStruct<Ok = JsValue, Error = Error>> ser::SerializeTupleVariant
    for VariantSerializer<S>
{
    type Ok = JsValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.inner.serialize_field(value)
    }

    fn end(self) -> Result {
        self.end(S::end)
    }
}

impl<S: ser::SerializeStruct<Ok = JsValue, Error = Error>> ser::SerializeStructVariant
    for VariantSerializer<S>
{
    type Ok = JsValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        self.inner.serialize_field(key, value)
    }

    fn end(self) -> Result {
        self.end(S::end)
    }
}

/// Serializes Rust iterables and tuples into JS arrays.
pub struct ArraySerializer<'s> {
    serializer: &'s Serializer,
    target: Array,
    idx: u32,
}

impl<'s> ArraySerializer<'s> {
    fn new(serializer: &'s Serializer) -> Self {
        Self {
            serializer,
            target: Array::new(),
            idx: 0,
        }
    }
}

impl ser::SerializeSeq for ArraySerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        self.target
            .set_ref(self.idx, &value.serialize(self.serializer)?);
        self.idx += 1;
        Ok(())
    }

    fn end(self) -> Result {
        Ok(self.target.into())
    }
}

impl ser::SerializeTuple for ArraySerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for ArraySerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result {
        ser::SerializeTuple::end(self)
    }
}

pub enum MapResult {
    Map(Map),
    Object(Object),
}

/// Serializes Rust maps into JS `Map` or plain JS objects.
///
/// Plain JS objects are used if `serialize_maps_as_objects` is set to `true`,
/// but then only string keys are supported.
pub struct MapSerializer<'s> {
    serializer: &'s Serializer,
    target: MapResult,
    next_key: Option<JsValue>,
}

impl<'s> MapSerializer<'s> {
    pub fn new(serializer: &'s Serializer, as_object: bool) -> Self {
        Self {
            serializer,
            target: if as_object {
                MapResult::Object(Object::new())
            } else {
                MapResult::Map(Map::new())
            },
            next_key: None,
        }
    }
}

impl ser::SerializeMap for MapSerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<()> {
        debug_assert!(self.next_key.is_none());
        self.next_key = Some(key.serialize(self.serializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self.next_key.take().unwrap_throw();
        let value_ser = value.serialize(self.serializer)?;
        match &self.target {
            MapResult::Map(map) => {
                map.set(&key, &value_ser);
            }
            MapResult::Object(object) => {
                let key = key.dyn_into::<JsString>().map_err(|_| {
                    Error::custom("Map key is not a string and cannot be an object key")
                })?;
                object.unchecked_ref::<ObjectExt>().set(key, value_ser);
            }
        }
        Ok(())
    }

    fn end(self) -> Result {
        debug_assert!(self.next_key.is_none());
        match self.target {
            MapResult::Map(map) => Ok(map.into()),
            MapResult::Object(object) => Ok(object.into()),
        }
    }
}

/// Serializes Rust structs into plain JS objects.
pub struct ObjectSerializer<'s> {
    serializer: &'s Serializer,
    target: ObjectExt,
}

impl<'s> ObjectSerializer<'s> {
    pub fn new(serializer: &'s Serializer) -> Self {
        Self {
            serializer,
            target: Object::new().unchecked_into::<ObjectExt>(),
        }
    }
}

impl ser::SerializeStruct for ObjectSerializer<'_> {
    type Ok = JsValue;
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<()> {
        let value = value.serialize(self.serializer)?;
        self.target.set(static_str_to_js(key), value);
        Ok(())
    }

    fn end(self) -> Result {
        Ok(self.target.into())
    }
}

/// A [`serde::Serializer`] that converts supported Rust values into a [`JsValue`].
#[derive(Default)]
pub struct Serializer {
    serialize_missing_as_null: bool,
    serialize_maps_as_objects: bool,
    serialize_large_number_types_as_bigints: bool,
    serialize_bytes_as_arrays: bool,
}

impl Serializer {
    /// Creates a new default [`Serializer`].
    pub const fn new() -> Self {
        Self {
            serialize_missing_as_null: false,
            serialize_maps_as_objects: false,
            serialize_large_number_types_as_bigints: false,
            serialize_bytes_as_arrays: false,
        }
    }

    /// Creates a JSON compatible serializer. This uses null instead of undefined, and
    /// uses plain objects instead of ES maps. So you will get the same result of
    /// `JsValue::from_serde`, and you can stringify results to JSON and store
    /// it without data loss.
    pub const fn json_compatible() -> Self {
        Self {
            serialize_missing_as_null: true,
            serialize_maps_as_objects: true,
            serialize_large_number_types_as_bigints: false,
            serialize_bytes_as_arrays: true,
        }
    }

    /// Set to `true` to serialize `()`, unit structs and `Option::None` to `null`
    /// instead of `undefined` in JS. `false` by default.
    pub const fn serialize_missing_as_null(mut self, value: bool) -> Self {
        self.serialize_missing_as_null = value;
        self
    }

    /// Set to `true` to serialize maps into plain JavaScript objects instead of
    /// ES2015 `Map`s. `false` by default.
    pub const fn serialize_maps_as_objects(mut self, value: bool) -> Self {
        self.serialize_maps_as_objects = value;
        self
    }

    /// Set to `true` to serialize 64-bit numbers to JavaScript `BigInt` instead of
    /// plain numbers. `false` by default.
    pub const fn serialize_large_number_types_as_bigints(mut self, value: bool) -> Self {
        self.serialize_large_number_types_as_bigints = value;
        self
    }

    /// Set to `true` to serialize bytes into plain JavaScript arrays instead of
    /// ES2015 `Uint8Array`s. `false` by default.
    pub const fn serialize_bytes_as_arrays(mut self, value: bool) -> Self {
        self.serialize_bytes_as_arrays = value;
        self
    }
}

macro_rules! forward_to_into {
    ($($name:ident($ty:ty);)*) => {
        $(fn $name(self, v: $ty) -> Result {
            Ok(v.into())
        })*
    };
}

impl<'s> ser::Serializer for &'s Serializer {
    type Ok = JsValue;
    type Error = Error;

    type SerializeSeq = ArraySerializer<'s>;
    type SerializeTuple = ArraySerializer<'s>;
    type SerializeTupleStruct = ArraySerializer<'s>;
    type SerializeTupleVariant = VariantSerializer<ArraySerializer<'s>>;
    type SerializeMap = MapSerializer<'s>;
    type SerializeStruct = ObjectSerializer<'s>;
    type SerializeStructVariant = VariantSerializer<ObjectSerializer<'s>>;

    forward_to_into! {
        serialize_bool(bool);

        serialize_i8(i8);
        serialize_i16(i16);
        serialize_i32(i32);

        serialize_u8(u8);
        serialize_u16(u16);
        serialize_u32(u32);

        serialize_f32(f32);
        serialize_f64(f64);

        serialize_str(&str);
    }

    /// Serializes `i64` into a `BigInt` or a JS number.
    ///
    /// If `serialize_large_number_types_as_bigints` is set to `false`,
    /// `i64` is serialized as a JS number. But in this mode only numbers
    /// within the safe integer range are supported.
    fn serialize_i64(self, v: i64) -> Result {
        if self.serialize_large_number_types_as_bigints {
            return Ok(v.into());
        }

        // Note: don't try to "simplify" by using `.abs()` as it can overflow,
        // but range check can't.
        const MIN_SAFE_INTEGER: i64 = Number::MIN_SAFE_INTEGER as i64;
        const MAX_SAFE_INTEGER: i64 = Number::MAX_SAFE_INTEGER as i64;

        if (MIN_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&v) {
            self.serialize_f64(v as _)
        } else {
            Err(Error::custom(format_args!(
                "{} can't be represented as a JavaScript number",
                v
            )))
        }
    }

    /// Serializes `u64` into a `BigInt` or a JS number.
    ///
    /// If `serialize_large_number_types_as_bigints` is set to `false`,
    /// `u64` is serialized as a JS number. But in this mode only numbers
    /// within the safe integer range are supported.
    fn serialize_u64(self, v: u64) -> Result {
        if self.serialize_large_number_types_as_bigints {
            return Ok(v.into());
        }

        if v <= Number::MAX_SAFE_INTEGER as u64 {
            self.serialize_f64(v as _)
        } else {
            Err(Error::custom(format_args!(
                "{} can't be represented as a JavaScript number",
                v
            )))
        }
    }

    /// Serializes `i128` into a `BigInt`.
    fn serialize_i128(self, v: i128) -> Result {
        Ok(JsValue::from(v))
    }

    /// Serializes `u128` into a `BigInt`.
    fn serialize_u128(self, v: u128) -> Result {
        Ok(JsValue::from(v))
    }

    /// Serializes `char` into a JS string.
    fn serialize_char(self, v: char) -> Result {
        Ok(JsString::from(v).into())
    }

    /// Serializes `bytes` into a JS `Uint8Array` or a plain JS array.
    ///
    /// If `serialize_bytes_as_arrays` is set to `true`, bytes are serialized as plain JS arrays.
    fn serialize_bytes(self, v: &[u8]) -> Result {
        // Create a `Uint8Array` view into a Rust slice, and immediately copy it to the JS memory.
        //
        // This is necessary because any allocation in WebAssembly can require reallocation of the
        // backing memory, which will invalidate existing views (including `Uint8Array`).
        let view = unsafe { Uint8Array::view(v) };
        if self.serialize_bytes_as_arrays {
            Ok(JsValue::from(
                Array::from_iterable(view.unchecked_ref::<Array>()).unwrap_throw(),
            ))
        } else {
            Ok(JsValue::from(Uint8Array::new(view.as_ref())))
        }
    }

    /// Serializes `None` into `undefined` or `null`.
    ///
    /// If `serialize_missing_as_null` is set to `true`, `None` is serialized as `null`.
    fn serialize_none(self) -> Result {
        self.serialize_unit()
    }

    /// Serializes `Some(T)` as `T`.
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result {
        value.serialize(self)
    }

    /// Serializes `()` into `undefined` or `null`.
    ///
    /// If `serialize_missing_as_null` is set to `true`, `()` is serialized as `null`.
    fn serialize_unit(self) -> Result {
        Ok(if self.serialize_missing_as_null {
            JsValue::NULL
        } else {
            JsValue::UNDEFINED
        })
    }

    /// Serializes unit structs into `undefined` or `null`.
    fn serialize_unit_struct(self, _name: &'static str) -> Result {
        self.serialize_unit()
    }

    /// For compatibility with serde-json, serializes unit variants as "Variant" strings.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result {
        Ok(static_str_to_js(variant).into())
    }

    /// Serializes newtype structs as their inner values.
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result {
        if name == PRESERVED_VALUE_MAGIC {
            let abi = value.serialize(self)?.unchecked_into_f64() as u32;
            // `PreservedValueSerWrapper` gives us ABI of a reference to a `JsValue` that is
            // guaranteed to be alive only during this call.
            // We must clone it before giving away the value to the caller.
            return Ok(unsafe { JsValue::ref_from_abi(abi) }.as_ref().clone());
        }
        value.serialize(self)
    }

    /// Serializes newtype variants as their inner values.
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result {
        VariantSerializer::new(variant, self.serialize_newtype_struct(variant, value)?).end(Ok)
    }

    /// Serializes any Rust iterable as a JS Array.
    // TODO: Figure out if there is a way to detect and serialize `Set` differently.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(ArraySerializer::new(self))
    }

    /// Serializes Rust tuples as JS arrays.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    /// Serializes Rust tuple structs as JS arrays.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_tuple(len)
    }

    /// Serializes Rust tuple variants as `{"Variant": [ ...tuple... ]}`.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(VariantSerializer::new(
            variant,
            self.serialize_tuple_struct(variant, len)?,
        ))
    }

    /// Serializes Rust maps into JS `Map` or plain JS objects.
    ///
    /// See [`MapSerializer`] for more details.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(MapSerializer::new(self, self.serialize_maps_as_objects))
    }

    /// Serializes Rust typed structs into plain JS objects.
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(ObjectSerializer::new(self))
    }

    /// Serializes Rust struct-like variants into `{"Variant": { ...fields... }}`.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(VariantSerializer::new(
            variant,
            self.serialize_struct(variant, len)?,
        ))
    }
}
