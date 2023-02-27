#[allow(clippy::module_inception)]
mod body;
mod to_bytes;
mod wasm;

pub use body::Body;
pub use to_bytes::to_bytes;
pub(crate) use wasm::WasmStreamBody;

pub use bytes::{Buf, BufMut, Bytes};

fn _assert_send() {
    fn _assert_send<T: Send>() {}

    _assert_send::<Body>();
}
