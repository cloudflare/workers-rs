use bytes::{Buf, BufMut, Bytes};

use crate::body::HttpBody;

// Copied from hyper under the following license:
// Copyright (c) 2014-2021 Sean McArthur
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
/// Concatenate the buffers from a body into a single `Bytes` asynchronously.
///
/// This may require copying the data into a single buffer.
///
/// # Note
///
/// Care needs to be taken if the remote is untrusted. The function doesn't implement any length
/// checks and an malicious peer might make it consume arbitrary amounts of memory. Checking the
/// `Content-Length` is a possibility, but it is not strictly mandated to be present.
pub async fn to_bytes<T>(body: T) -> Result<Bytes, T::Error>
where
    T: HttpBody,
{
    futures_util::pin_mut!(body);

    // If there's only 1 chunk, we can just return Buf::to_bytes()
    let mut first = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(Bytes::new());
    };

    let second = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(first.copy_to_bytes(first.remaining()));
    };

    // Don't pre-emptively reserve *too* much.
    let rest = (body.size_hint().lower() as usize).min(1024 * 16);
    let cap = first
        .remaining()
        .saturating_add(second.remaining())
        .saturating_add(rest);
    // With more than 1 buf, we gotta flatten into a Vec first.
    let mut vec = Vec::with_capacity(cap);
    vec.put(first);
    vec.put(second);

    while let Some(buf) = body.data().await {
        vec.put(buf?);
    }

    Ok(vec.into())
}
