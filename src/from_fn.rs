//! create initializers from functions/closures

use crate::{Init, Initializer, Uninit};

/// Create an initializer from a function/closure
pub const fn from_fn<T: ?Sized>(
    f: impl FnOnce(Uninit<T>) -> Init<T>,
) -> impl Initializer<T, Error = core::convert::Infallible> {
    struct FromFn<F>(F);

    impl<T: ?Sized, F: FnOnce(Uninit<T>) -> Init<T>> Initializer<T> for FromFn<F> {
        type Error = core::convert::Infallible;

        fn try_init_into(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
            Ok((self.0)(ptr))
        }
    }

    FromFn(f)
}

/// Create an initializer from a function/closure
pub const fn try_from_fn<T: ?Sized, E>(
    f: impl FnOnce(Uninit<T>) -> Result<Init<T>, E>,
) -> impl Initializer<T, Error = E> {
    struct FromFn<F>(F);

    impl<T: ?Sized, E, F: FnOnce(Uninit<T>) -> Result<Init<T>, E>> Initializer<T> for FromFn<F> {
        type Error = E;

        fn try_init_into(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
            (self.0)(ptr)
        }
    }

    FromFn(f)
}
