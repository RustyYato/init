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

#[doc(hidden)]
pub use core;

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

mod macros;

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

/// Convert a value to an initializer for that value to work around the missing (and
/// unimplementable) T: Ctor<T>,
#[inline]
pub fn init<T>(value: T) -> CtorFromValue<T> {
    CtorFromValue(value)
}

/// An adapter type to convert a value to an initializer, see [`init`] for details
#[derive(Clone, Copy)]
pub struct CtorFromValue<F>(F);

impl<T> Ctor<CtorFromValue<Self>> for T {
    type Error = core::convert::Infallible;

    fn try_init(
        ptr: ptr::Uninit<Self>,
        CtorFromValue(args): CtorFromValue<Self>,
    ) -> Result<ptr::Init<Self>, Self::Error> {
        Ok(ptr.write(args))
    }
}

/// Convert a closure to an initializer
#[inline]
pub fn init_fn<T: ?Sized, F: FnOnce(ptr::Uninit<T>) -> ptr::Init<T>>(f: F) -> CtorFromFn<F> {
    CtorFromFn(f)
}

/// An adapter type to convert a closure to an initializer, see [`init_fn`] for details
#[derive(Clone, Copy)]
pub struct CtorFromFn<F>(F);

impl<T: ?Sized, F: FnOnce(ptr::Uninit<T>) -> ptr::Init<T>> Ctor<CtorFromFn<F>> for T {
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
pub fn try_init_fn<T: ?Sized, E, F: FnOnce(ptr::Uninit<T>) -> Result<ptr::Init<T>, E>>(
    f: F,
) -> TryCtorFromFn<F> {
    TryCtorFromFn(f)
}

/// An adapter type to convert a closure to an initializer, see [`try_init_fn`] for details
pub struct TryCtorFromFn<F>(F);

impl<T: ?Sized, E, F: FnOnce(ptr::Uninit<T>) -> Result<ptr::Init<T>, E>> Ctor<TryCtorFromFn<F>>
    for T
{
    type Error = E;

    fn try_init(
        ptr: ptr::Uninit<Self>,
        TryCtorFromFn(args): TryCtorFromFn<F>,
    ) -> Result<ptr::Init<Self>, Self::Error> {
        args(ptr)
    }
}

/// Initialize and return the value `T` on the stack
pub fn try_init_on_stack<T: Unpin, I: crate::Initializer<T>>(init: I) -> Result<T, I::Error> {
    let mut value = core::mem::MaybeUninit::uninit();
    // SAFETY: value.as_mut_ptr() is a non-null, aligned, allocated for T, and not aliased
    let uninit = unsafe { ptr::Uninit::from_raw(value.as_mut_ptr()) };
    uninit.try_init(init)?.take_ownership();
    // SAFETY: the value was initialized
    Ok(unsafe { value.assume_init() })
}

/// Initialize and return the value `T` on the stack
pub fn try_init_with<T, I: crate::Initializer<T>, R>(
    init: I,
    f: impl FnOnce(core::pin::Pin<&mut T>) -> R,
) -> Result<R, I::Error> {
    let mut value = core::mem::MaybeUninit::uninit();
    // SAFETY: value.as_mut_ptr() is a non-null, aligned, allocated for T, and not aliased
    let uninit = unsafe { ptr::Uninit::from_raw(value.as_mut_ptr()) };
    let mut init = uninit.try_init(init)?;
    // SAFETY: the init will be dropped at the end of this scope, before this stack frame is deallocated
    Ok(f(unsafe { init.get_pin_mut_unchecked() }))
}
