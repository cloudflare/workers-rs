use crate::EnvBinding;
use crate::Result;
use js_sys::Array;
use js_sys::Object;
use std::marker::PhantomData;
use wasm_bindgen::{JsCast, JsValue};
use worker_sys::AnalyticsEngineDataset as AnalyticsEngineSys;

#[derive(Clone)]
pub struct AnalyticsEngineDataset(AnalyticsEngineSys);

unsafe impl Send for AnalyticsEngineDataset {}
unsafe impl Sync for AnalyticsEngineDataset {}

impl EnvBinding for AnalyticsEngineDataset {
    const TYPE_NAME: &'static str = "AnalyticsEngineDataset";

    // Override get to perform an unchecked cast from an object.
    // Miniflare defines the binding as an Object and not a class so its name is not available for checking.
    // https://github.com/cloudflare/workers-sdk/blob/main/packages/wrangler/templates/middleware/middleware-mock-analytics-engine.ts#L6
    fn get(val: JsValue) -> Result<Self> {
        let obj = Object::from(val);
        Ok(obj.unchecked_into())
    }
}

impl JsCast for AnalyticsEngineDataset {
    fn instanceof(val: &JsValue) -> bool {
        val.is_instance_of::<AnalyticsEngineDataset>()
    }

    fn unchecked_from_js(val: JsValue) -> Self {
        Self(val.into())
    }

    fn unchecked_from_js_ref(val: &JsValue) -> &Self {
        unsafe { &*(val as *const JsValue as *const Self) }
    }
}

impl From<AnalyticsEngineDataset> for JsValue {
    fn from(analytics_engine: AnalyticsEngineDataset) -> Self {
        JsValue::from(analytics_engine.0)
    }
}

impl AsRef<JsValue> for AnalyticsEngineDataset {
    fn as_ref(&self) -> &JsValue {
        &self.0
    }
}

impl AnalyticsEngineDataset {
    pub fn write_data_point<T>(&self, event: &AnalyticsEngineDataPoint<T>) -> Result<()>
    where
        T: BlobType,
        AnalyticsEngineDataPoint<T>: Clone,
        JsValue: From<AnalyticsEngineDataPoint<T>>,
    {
        Ok(self.0.write_data_point(event.to_js_object()?)?)
    }
}

// Marker traits to constrain T
pub trait BlobType {}
impl BlobType for String {}
impl BlobType for Vec<u8> {}

#[derive(Clone, Debug)]
pub struct AnalyticsEngineDataPoint<T>
where
    T: BlobType,
{
    indexes: Vec<String>,
    doubles: Vec<f64>,
    blobs: Vec<T>,
}

pub struct AnalyticsEngineDataPointBuilder<T: BlobType> {
    indexes: Vec<String>,
    doubles: Vec<f64>,
    blobs: Vec<T>,
    _phantom: PhantomData<T>,
}

impl<T: BlobType> AnalyticsEngineDataPointBuilder<T> {
    pub fn new() -> Self {
        Self {
            indexes: Vec::new(),
            doubles: Vec::new(),
            blobs: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub fn indexes(mut self, index: impl Into<Vec<String>>) -> Self {
        self.indexes = index.into();
        self
    }

    pub fn doubles(mut self, doubles: impl Into<Vec<f64>>) -> Self {
        self.doubles = doubles.into();
        self
    }

    pub fn blobs(mut self, blobs: impl Into<Vec<T>>) -> Self {
        self.blobs = blobs.into();
        self
    }

    pub fn build(self) -> AnalyticsEngineDataPoint<T> {
        AnalyticsEngineDataPoint {
            indexes: self.indexes,
            doubles: self.doubles,
            blobs: self.blobs,
        }
    }
}

// Implement From for JsValue separately for each type
impl From<AnalyticsEngineDataPoint<String>> for JsValue {
    fn from(event: AnalyticsEngineDataPoint<String>) -> Self {
        let obj = Object::new();

        let indexes = Array::new();
        for index in event.indexes {
            indexes.push(&JsValue::from_str(&index));
        }
        js_sys::Reflect::set(&obj, &JsValue::from_str("indexes"), &indexes).unwrap();

        let doubles = Array::new();
        for double in event.doubles {
            doubles.push(&JsValue::from_f64(double));
        }
        js_sys::Reflect::set(&obj, &JsValue::from_str("doubles"), &doubles).unwrap();

        let blobs = Array::new();
        for blob in event.blobs {
            blobs.push(&JsValue::from_str(&blob));
        }
        js_sys::Reflect::set(&obj, &JsValue::from_str("blobs"), &blobs).unwrap();

        JsValue::from(obj)
    }
}

impl From<AnalyticsEngineDataPoint<Vec<u8>>> for JsValue {
    fn from(event: AnalyticsEngineDataPoint<Vec<u8>>) -> Self {
        let obj = Object::new();

        let indexes = Array::new();
        for index in event.indexes {
            indexes.push(&JsValue::from_str(&index));
        }
        js_sys::Reflect::set(&obj, &JsValue::from_str("indexes"), &indexes).unwrap();

        let doubles = Array::new();
        for double in event.doubles {
            doubles.push(&JsValue::from_f64(double));
        }
        js_sys::Reflect::set(&obj, &JsValue::from_str("doubles"), &doubles).unwrap();

        // An array of u8 arrays.
        let blobs = Array::new();
        for blob in event.blobs {
            let slice = Array::new();
            for byte in blob {
                slice.push(&JsValue::from_f64(byte as f64));
            }
            blobs.push(&JsValue::from(slice));
        }
        js_sys::Reflect::set(&obj, &JsValue::from_str("blobs"), &blobs).unwrap();

        JsValue::from(obj)
    }
}

impl<T: BlobType> AnalyticsEngineDataPoint<T> {
    pub fn to_js_object(&self) -> Result<JsValue>
    where
        Self: Clone,
        JsValue: From<Self>,
    {
        Ok(self.clone().into())
    }
}
