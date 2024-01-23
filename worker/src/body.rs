//! Body types and functions

#[allow(clippy::module_inception)]
mod body;
mod to_bytes;
mod wasm;

pub use body::Body;
pub(crate) use body::BodyInner;
pub(crate) use body::BoxBodyReader;
pub use http_body::Body as HttpBody;
pub use to_bytes::to_bytes;

pub use bytes::{Buf, BufMut, Bytes};

fn _assert_send() {
    use crate::futures::assert_send;
    assert_send::<Body>();
}
