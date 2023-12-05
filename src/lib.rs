#![no_std]
#![forbid(
    missing_docs,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks
)]

//! # `init`
//!
//! A crate to handle in-place initialization to support initializing unsized or pinned types.

pub mod ptr;
