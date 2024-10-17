#![no_std]
#![forbid(
    unsafe_op_in_unsafe_fn,
    missing_docs,
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::alloc_instead_of_core,
    clippy::missing_const_for_fn,
    clippy::missing_const_for_thread_local
)]

//! A crate for in-place initialization of values for performance and safety

mod polyfill;
mod ptr;

pub use ptr::{Init, Uninit};
