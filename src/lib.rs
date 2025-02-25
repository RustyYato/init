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

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[doc(hidden)]
#[macro_use]
#[path = "macros.rs"]
pub mod __private_macros;

mod polyfill;
mod ptr;

pub mod array;
#[cfg(feature = "alloc")]
pub mod boxed;
pub mod from_fn;
pub mod layout_provider;
pub mod slice;

mod primitive;

pub mod thin;

pub mod slice_writer;

pub use from_fn::{from_fn, try_from_fn};
pub use primitive::PrimitiveLayoutProvider;

pub use ptr::{Init, Uninit};

/// A constructor trait, specifies how to initialize a `T` in place
///
/// To be implemented on the host type
pub trait Ctor<Args = ()> {
    /// The error type in case initialization fails
    type Error;

    /// initialize self in place
    fn try_init(ptr: Uninit<Self>, args: Args) -> Result<Init<Self>, Self::Error>;
}

/// An initializer trait, specifies how to initialize a `T` in place
///
/// To be implemented on the argument type to initialize with. This allows 3rd party initializers
pub trait Initializer<T: ?Sized> {
    /// The error type in case initialization fails
    type Error;

    /// initialize ptr in place
    fn try_init_into(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error>;
}

impl<T: ?Sized, Args: Initializer<T>> Ctor<Args> for T {
    type Error = Args::Error;

    fn try_init(ptr: Uninit<Self>, args: Args) -> Result<Init<Self>, Self::Error> {
        args.try_init_into(ptr)
    }
}
