use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::HashMap,
};

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/js/alloc.js")]
extern "C" {
    #[wasm_bindgen(js_name = onAlloc)]
    fn on_alloc(pointer: usize, size: usize);

    #[wasm_bindgen(js_name = onDealloc)]
    fn on_dealloc(pointer: usize);
}

static mut IGNORE: bool = false;

pub struct DebugAllocator<A: GlobalAlloc>(pub A);

impl<A: GlobalAlloc> DebugAllocator<A> {
    fn usage() -> HashMap<String, ()> {
        // SAFETY: Since we only every have a single thread we don't need to worry about
        // concurrent access to the IGNORE variable.
        unsafe { IGNORE = true };

        // SAFETY: Since we only every have a single thread we don't need to worry about
        // concurrent access to the IGNORE variable.
        unsafe { IGNORE = false };

        todo!()
    }
}

impl DebugAllocator<System> {
    /// Create a new instance of the [System] allocator.
    pub fn system() -> Self {
        Self(System)
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for DebugAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.0.alloc(layout);
        on_alloc(ptr as usize, layout.size());
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        on_dealloc(ptr as usize);
        self.0.dealloc(ptr, layout);
    }
}
