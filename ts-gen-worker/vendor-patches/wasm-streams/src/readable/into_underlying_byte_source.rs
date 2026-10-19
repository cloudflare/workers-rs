use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, RefUnwindSafe, UnwindSafe};
use std::pin::Pin;
use std::rc::Rc;

use futures_util::future::{AbortHandle, TryFutureExt, abortable};
use futures_util::io::{AsyncRead, AsyncReadExt};
use js_sys::{Error as JsError, Promise, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::util::{checked_cast_to_u32, clamp_to_usize};

use super::sys;

#[wasm_bindgen(private)]
pub(crate) struct IntoUnderlyingByteSource {
    inner: Rc<RefCell<Inner>>,
    default_buffer_len: usize,
    controller: Option<sys::ReadableByteStreamController>,
    pull_handle: Option<AbortHandle>,
}

// SAFETY: `Inner` holds an `Option<Pin<Box<dyn AsyncRead>>>` and uses the
// take-and-replace pattern across every fallible await point in `Inner::pull`.
// On panic, the inner async_read is already taken out of the `Option`, leaving
// the cell in a clean `None` state. The `Rc<RefCell<Inner>>` interior mutability
// therefore cannot expose torn invariants to a subsequent call after a caught
// panic, satisfying the logical unwind-safety contract enforced by
// `#[wasm_bindgen]` exports under `panic = "unwind"`.
impl UnwindSafe for IntoUnderlyingByteSource {}
impl RefUnwindSafe for IntoUnderlyingByteSource {}

impl IntoUnderlyingByteSource {
    pub fn new(async_read: Box<dyn AsyncRead>, default_buffer_len: usize) -> Self {
        IntoUnderlyingByteSource {
            inner: Rc::new(RefCell::new(Inner::new(async_read))),
            default_buffer_len,
            controller: None,
            pull_handle: None,
        }
    }
}

#[allow(clippy::await_holding_refcell_ref)]
#[wasm_bindgen]
impl IntoUnderlyingByteSource {
    #[wasm_bindgen(getter, js_name = type)]
    pub fn type_(&self) -> sys::ReadableStreamType {
        sys::ReadableStreamType::Bytes
    }

    #[wasm_bindgen(getter, js_name = autoAllocateChunkSize)]
    pub fn auto_allocate_chunk_size(&self) -> usize {
        self.default_buffer_len
    }

    pub fn start(&mut self, controller: sys::ReadableByteStreamController) {
        self.controller = Some(controller);
    }

    pub fn pull(&mut self, controller: sys::ReadableByteStreamController) -> Promise {
        let inner = self.inner.clone();
        let fut = async move {
            // This mutable borrow can never panic, since the ReadableStream always queues
            // each operation on the underlying source.
            let mut inner = inner.try_borrow_mut().unwrap_throw();
            inner.pull(controller).await
        };

        // Allow aborting the future from cancel().
        let (fut, handle) = abortable(fut);
        // Ignore errors from aborting the future.
        let fut = fut.unwrap_or_else(|_| Ok(JsValue::undefined()));

        self.pull_handle = Some(handle);
        // SAFETY: We use the take-and-replace pattern in Inner::pull() to ensure
        // that if a panic occurs, the async_read is already taken out of the Option,
        // leaving it in a clean None state. This prevents use of corrupted state
        // after a panic is caught.
        future_to_promise(AssertUnwindSafe(fut))
    }

    pub fn cancel(self) {
        // The stream has been canceled, drop everything.
        drop(self);
    }
}

impl Drop for IntoUnderlyingByteSource {
    fn drop(&mut self) {
        // Abort the pending pull, if any.
        if let Some(handle) = self.pull_handle.take() {
            handle.abort();
        }
    }
}

struct Inner {
    async_read: Option<Pin<Box<dyn AsyncRead>>>,
    buffer: Vec<u8>,
}

impl Inner {
    fn new(async_read: Box<dyn AsyncRead>) -> Self {
        Inner {
            async_read: Some(async_read.into()),
            buffer: Vec::new(),
        }
    }

    async fn pull(
        &mut self,
        controller: sys::ReadableByteStreamController,
    ) -> Result<JsValue, JsValue> {
        // We set autoAllocateChunkSize, so there should always be a BYOB request.
        let request = controller.byob_request().unwrap_throw();
        // Resize the buffer to fit the BYOB request.
        let request_view = request.view().unwrap_throw().unchecked_into::<Uint8Array>();
        let request_len = clamp_to_usize(request_view.byte_length());
        if self.buffer.len() < request_len {
            self.buffer.resize(request_len, 0);
        }

        // Take the async_read out before the fallible/panickable operation.
        // This ensures that if a panic occurs, self.async_read is already None,
        // so any subsequent call will fail cleanly instead of using corrupted state.
        let mut async_read = self.async_read.take().unwrap_throw();

        match async_read.read(&mut self.buffer[0..request_len]).await {
            Ok(0) => {
                // Stream closed: don't put it back, clear buffer, close controller
                self.buffer = Vec::new();
                controller.close()?;
                request.respond_with_u32(0)?;
            }
            Ok(bytes_read) => {
                // Success: put the async_read back for reuse
                self.async_read = Some(async_read);
                // Copy read bytes from buffer to BYOB request view
                debug_assert!(bytes_read <= request_len);
                let bytes_read_u32 = checked_cast_to_u32(bytes_read);
                let dest = Uint8Array::new_with_byte_offset_and_length(
                    &request_view.buffer(),
                    request_view.byte_offset(),
                    bytes_read_u32,
                );
                dest.copy_from(&self.buffer[0..bytes_read]);
                // Respond to BYOB request
                request.respond_with_u32(bytes_read_u32)?;
            }
            Err(err) => {
                // Error: don't put it back, clear buffer, return error
                self.buffer = Vec::new();
                return Err(JsError::new(&err.to_string()).into());
            }
        };
        // Panic: async_read is dropped during unwind, self.async_read remains None
        Ok(JsValue::undefined())
    }
}
