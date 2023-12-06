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

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod iter;
pub mod ptr;

pub mod slice_writer;

pub mod layout_provider;

mod primitive;
pub mod slice;

#[cfg(feature = "alloc")]
pub mod boxed;

/// A trait to initialize a location in place, or error
pub trait Ctor<Args = ()> {
    /// The error type if initialization fails, use [`core::convert::Infallible`] if initialization can't fail
    type Error;

    /// Try to initialize the location `ptr` with `args` or error if not possible
    fn try_init(ptr: ptr::Uninit<Self>, args: Args) -> Result<ptr::Init<Self>, Self::Error>;
}

/// A trait to initialize a location in place, or error
///
/// If possible, implement `Ctor` instead
pub trait Initializer<T: ?Sized> {
    /// The error type if initialization fails, use [`core::convert::Infallible`] if initialization can't fail
    type Error;

    /// Try to initialize the location `ptr` with `self` or error if not possible
    fn try_init_into(self, ptr: ptr::Uninit<T>) -> Result<ptr::Init<T>, Self::Error>;
}

impl<T: ?Sized + Ctor<A>, A> Initializer<T> for A {
    type Error = T::Error;

    fn try_init_into(self, ptr: ptr::Uninit<T>) -> Result<ptr::Init<T>, Self::Error> {
        T::try_init(ptr, self)
    }
}

/// Convert a closure to an initializer
#[inline]
pub fn init_fn<T, F: FnOnce(ptr::Uninit<T>) -> ptr::Init<T>>(f: F) -> CtorFromFn<F> {
    CtorFromFn(f)
}

/// An adapter type to convert a closure to an initializer, see [`init_fn`] for details
pub struct CtorFromFn<F>(F);

impl<T, F: FnOnce(ptr::Uninit<T>) -> ptr::Init<T>> Ctor<CtorFromFn<F>> for T {
    type Error = core::convert::Infallible;

    fn try_init(
        ptr: ptr::Uninit<Self>,
        CtorFromFn(args): CtorFromFn<F>,
    ) -> Result<ptr::Init<Self>, Self::Error> {
        Ok(args(ptr))
    }
}

/// Convert a closure to an initializer
#[inline]
pub fn try_init_fn<T, E, F: FnOnce(ptr::Uninit<T>) -> Result<ptr::Init<T>, E>>(
    f: F,
) -> TryCtorFromFn<F> {
    TryCtorFromFn(f)
}

/// An adapter type to convert a closure to an initializer, see [`try_init_fn`] for details
pub struct TryCtorFromFn<F>(F);

impl<T, E, F: FnOnce(ptr::Uninit<T>) -> Result<ptr::Init<T>, E>> Ctor<TryCtorFromFn<F>> for T {
    type Error = E;

    fn try_init(
        ptr: ptr::Uninit<Self>,
        TryCtorFromFn(args): TryCtorFromFn<F>,
    ) -> Result<ptr::Init<Self>, Self::Error> {
        args(ptr)
    }
}
