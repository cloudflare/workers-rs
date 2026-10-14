//! Cloudflare Workers bindings generated from `types/worker.d.ts` by ts-gen.
//!
//! `src/bindings.rs` is generated; regenerate it with `chomp build:ts-gen-worker:types`
//! after editing the declarations.

#![allow(clippy::all, non_upper_case_globals)]

mod bindings;
mod error;

pub use bindings::*;
pub use error::{Error, Result};
