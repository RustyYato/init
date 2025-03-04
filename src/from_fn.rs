//! create initializers from functions/closures

use crate::{
    layout_provider::{DefaultLayoutProviderFor, SizedLayoutProvider},
    Init, Initializer, Uninit,
};

/// Converts a closure to an initializer
#[derive(Clone, Copy)]
pub struct InitFn<F>(F);

impl<T: ?Sized, F: FnOnce(Uninit<T>) -> Init<T>> Initializer<T> for InitFn<F> {
    type Error = core::convert::Infallible;

    fn try_init_into(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        Ok((self.0)(ptr))
    }
}

impl<F, T> DefaultLayoutProviderFor<T> for InitFn<F> {
    type LayoutProvider = SizedLayoutProvider;
}

impl<F, T> DefaultLayoutProviderFor<T> for TryInitFn<F> {
    type LayoutProvider = SizedLayoutProvider;
}

/// Create an initializer from a function/closure
pub const fn from_fn<T, F>(f: F) -> InitFn<F>
where
    T: ?Sized,
    F: FnOnce(Uninit<T>) -> Init<T>,
{
    InitFn(f)
}

/// Converts a closure to an initializer
#[derive(Clone, Copy)]
pub struct TryInitFn<F>(F);

impl<T: ?Sized, E, F: FnOnce(Uninit<T>) -> Result<Init<T>, E>> Initializer<T> for TryInitFn<F> {
    type Error = E;

    fn try_init_into(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        (self.0)(ptr)
    }
}

/// Create an initializer from a function/closure
pub const fn try_from_fn<T, E, F>(f: F) -> TryInitFn<F>
where
    T: ?Sized,
    F: FnOnce(Uninit<T>) -> Result<Init<T>, E>,
{
    TryInitFn(f)
}

/// Converts a closure to an initializer
#[derive(Clone, Copy)]
pub struct WithValue<T>(T);

impl<T> Initializer<T> for WithValue<T> {
    type Error = core::convert::Infallible;

    fn try_init_into(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        Ok(ptr.write(self.0))
    }
}

/// Create an initializer from a value
pub const fn with_value<T>(value: T) -> WithValue<T> {
    WithValue(value)
}
