use crate::EnvBinding;
use crate::Result;
use js_sys::Object;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::{JsCast, JsValue};
use worker_sys::AnalyticsEngineDataset as AnalyticsEngineSys;

#[derive(Debug, Clone)]
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
    pub fn write_data_point(&self, event: &AnalyticsEngineDataPoint) -> Result<()>
    where
        AnalyticsEngineDataPoint: Clone,
        JsValue: From<AnalyticsEngineDataPoint>,
    {
        Ok(self.0.write_data_point(event.to_js_object()?)?)
    }
}

#[derive(Debug)]
pub enum BlobType {
    String(String),
    Blob(Vec<u8>),
}

impl From<BlobType> for JsValue {
    fn from(val: BlobType) -> Self {
        match val {
            BlobType::String(s) => JsValue::from_str(&s),
            BlobType::Blob(b) => {
                let value = Uint8Array::from(b.as_slice());
                value.into()
            }
        }
    }
}

impl From<&str> for BlobType {
    fn from(value: &str) -> Self {
        BlobType::String(value.to_string())
    }
}

impl From<String> for BlobType {
    fn from(value: String) -> Self {
        BlobType::String(value)
    }
}

impl From<&[u8]> for BlobType {
    fn from(value: &[u8]) -> Self {
        BlobType::Blob(value.to_vec())
    }
}

impl From<Vec<u8>> for BlobType {
    fn from(value: Vec<u8>) -> Self {
        BlobType::Blob(value)
    }
}

impl<const COUNT: usize> From<&[u8; COUNT]> for BlobType {
    fn from(value: &[u8; COUNT]) -> Self {
        BlobType::Blob(value.to_vec())
    }
}

#[derive(Debug, Clone)]
pub struct AnalyticsEngineDataPoint {
    indexes: Array,
    doubles: Array,
    blobs: Array,
}

#[derive(Debug)]
pub struct AnalyticsEngineDataPointBuilder {
    indexes: Array,
    doubles: Array,
    blobs: Array,
}

impl AnalyticsEngineDataPointBuilder {
    pub fn new() -> Self {
        Self {
            indexes: Array::new(),
            doubles: Array::new(),
            blobs: Array::new(),
        }
    }

    /// Sets the index values for the data point.
    /// While the indexes field accepts an array, you currently must *only* provide a single index.
    /// If you attempt to provide multiple indexes, your data point will not be recorded.
    ///
    /// # Arguments
    ///
    /// * `index`: A string or byte-array value to use as the index.
    ///
    /// returns: AnalyticsEngineDataPointBuilder
    ///
    /// # Examples
    ///
    /// ```
    ///  use worker::AnalyticsEngineDataPointBuilder;
    ///
    ///  let data = AnalyticsEngineDataPointBuilder::new()
    ///     .indexes(["index1"])
    ///     .build();
    /// ```
    pub fn indexes<'index>(mut self, indexes: impl AsRef<[&'index str]>) -> Self {
        let values = Array::new();
        for idx in indexes.as_ref() {
            values.push(&JsValue::from_str(idx));
        }
        self.indexes = values;
        self
    }

    /// Adds a numeric value to the end of the array of doubles.
    ///
    /// # Arguments
    ///
    /// * `double`: The numeric values that you want to record in your data point
    ///
    /// returns: AnalyticsEngineDataPointBuilder
    ///
    /// # Examples
    ///
    /// ```
    ///  use worker::AnalyticsEngineDataPointBuilder;
    ///  let point = AnalyticsEngineDataPointBuilder::new()
    ///     .indexes(["index1"])
    ///     .add_double(25)     // double1
    ///     .add_double(0.5)    // double2
    ///     .build();
    ///  println!("{:?}", point);
    /// ```
    pub fn add_double(self, double: impl Into<f64>) -> Self {
        self.doubles.push(&JsValue::from_f64(double.into()));
        self
    }

    /// Set doubles1-20 with the provide values. This method will remove any doubles previously
    /// added using the `add_double` method.
    ///
    /// # Arguments
    ///
    /// * `doubles`: An array of doubles
    ///
    /// returns: AnalyticsEngineDataPointBuilder
    ///
    /// # Examples
    ///
    /// ```
    ///  use worker::AnalyticsEngineDataPointBuilder;
    ///  let point = AnalyticsEngineDataPointBuilder::new()
    ///     .indexes(["index1"])
    ///     .add_double(1) // value will be replaced by the following line
    ///     .doubles([1, 2, 3]) // sets double1, double2 and double3
    ///     .build();
    ///  println!("{:?}", point);
    /// ```
    pub fn doubles(mut self, doubles: impl IntoIterator<Item = f64>) -> Self {
        let values = Array::new();
        for n in doubles {
            values.push(&JsValue::from_f64(n));
        }
        self.doubles = values;
        self
    }

    /// Adds a blob-like value to the end of the array of blobs.
    ///
    /// # Arguments
    ///
    /// * `blob`: The blob values that you want to record in your data point
    ///
    /// returns: AnalyticsEngineDataPointBuilder
    ///
    /// # Examples
    ///
    /// ```
    ///  use worker::AnalyticsEngineDataPointBuilder;
    ///  let point = AnalyticsEngineDataPointBuilder::new()
    ///     .indexes(["index1"])
    ///     .add_blob("Seattle")            // blob1
    ///     .add_blob("USA")                // blob2
    ///     .add_blob("pro_sensor_9000")    // blob3
    ///     .build();
    ///  println!("{:?}", point);
    /// ```
    pub fn add_blob(self, blob: impl Into<BlobType>) -> Self {
        let v = blob.into();
        self.blobs.push(&v.into());
        self
    }

    /// Sets blobs1-20 with the provided array, replacing any values previously stored using `add_blob`.
    ///
    /// # Arguments
    ///
    /// * `blob`: The blob values that you want to record in your data point
    ///
    /// returns: AnalyticsEngineDataPointBuilder
    ///
    /// # Examples
    ///
    /// ```
    ///  use worker::AnalyticsEngineDataPointBuilder;
    ///  let point = AnalyticsEngineDataPointBuilder::new()
    ///     .indexes(["index1"])
    ///     .blobs(["Seattle", "USA", "pro_sensor_9000"]) // sets blob1, blob2, and blob3
    ///     .build();
    ///  println!("{:?}", point);
    /// ```
    pub fn blobs(mut self, blobs: impl IntoIterator<Item = impl Into<BlobType>>) -> Self {
        let values = Array::new();
        for blob in blobs {
            let value = blob.into();
            values.push(&value.into());
        }
        self.blobs = values;
        self
    }

    pub fn build(self) -> AnalyticsEngineDataPoint {
        AnalyticsEngineDataPoint {
            indexes: self.indexes,
            doubles: self.doubles,
            blobs: self.blobs,
        }
    }

    /// Write the data point to the provided analytics engine dataset. This is a convenience method
    /// that can be used in place of a `.build()` followed by a call to `dataset.write_data_point(point)`.
    ///
    /// # Arguments
    ///
    /// * `dataset`: Analytics engine dataset binding
    ///
    /// returns: worker::Result<()>
    ///
    /// # Examples
    ///
    /// ```
    ///  use worker::{Env, AnalyticsEngineDataPointBuilder, Response};
    ///  use std::io::Error;
    ///
    ///  fn main(env: Env) -> worker::Result<Response> {
    ///     let dataset = match env.analytics_engine("HTTP_ANALYTICS") {
    ///         Ok(dataset) => dataset,
    ///         Err(err) => return Response::error(format!("Failed to get dataset: {err:?}"), 500),
    ///     };
    ///
    ///     AnalyticsEngineDataPointBuilder::new()
    ///         .indexes(vec!["index1"].as_slice())
    ///         .add_blob("GET") // blob1
    ///         .add_double(200) // double1
    ///         .write_to(&dataset)?;
    ///
    ///     Response::ok("OK")
    /// }
    /// ```
    pub fn write_to(self, dataset: &AnalyticsEngineDataset) -> Result<()> {
        dataset.write_data_point(&self.build())
    }
}

// Implement From for JsValue separately for each type
impl From<AnalyticsEngineDataPoint> for JsValue {
    fn from(event: AnalyticsEngineDataPoint) -> Self {
        let obj = Object::new();

        js_sys::Reflect::set(&obj, &JsValue::from_str("indexes"), &event.indexes).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("doubles"), &event.doubles).unwrap();

        let blobs = Array::new();
        for blob in event.blobs {
            blobs.push(&JsValue::from(&blob));
        }
        js_sys::Reflect::set(&obj, &JsValue::from_str("blobs"), &blobs).unwrap();

        JsValue::from(obj)
    }
}

impl AnalyticsEngineDataPoint {
    pub fn to_js_object(&self) -> Result<JsValue>
    where
        Self: Clone,
        JsValue: From<Self>,
    {
        Ok(self.clone().into())
    }
}
