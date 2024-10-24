#![allow(clippy::cmp_null, ambiguous_wide_pointer_comparisons)]

use crate::{
    layout_provider::{DefaultLayoutProviderFor, LayoutProvider},
    Init, Uninit,
};

use core::{alloc::Layout, marker::PhantomData, ptr::NonNull};

/// A [`LayoutProvider`] for [`Sized`] types
pub struct PrimitiveLayoutProvider;

macro_rules! prim {
    ($(=> [$($binder:tt)*])? $t:ty => $zero:expr) => {
        impl<$($($binder)*)?> DefaultLayoutProviderFor<$t> for () {
            type LayoutProvider = PrimitiveLayoutProvider;
        }
        /// SAFETY: is_zeroed only returns false
        unsafe impl<$($($binder)*)?> LayoutProvider<$t, ()> for PrimitiveLayoutProvider {
            fn layout(_: &()) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            fn cast(ptr: NonNull<()>, _: &()) -> NonNull<$t> {
                ptr.cast()
            }

            fn is_zeroed(_args: &()) -> bool {
                true
            }
        }

        impl<$($($binder)*)?> crate::Initializer<$t> for () {
            type Error = core::convert::Infallible;

            fn try_init_into(self, u: Uninit<$t>) -> Result<Init<$t>, Self::Error> {
                Ok(u.write($zero))
            }
        }

        impl<$($($binder)*)?> DefaultLayoutProviderFor<$t> for $t {
            type LayoutProvider = PrimitiveLayoutProvider;
        }
        /// SAFETY: is_zeroed only returns true if args is zero
        unsafe impl<$($($binder)*)?> LayoutProvider<$t, $t> for PrimitiveLayoutProvider {
            fn layout(_: &$t) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            fn cast(ptr: NonNull<()>, _: &$t) -> NonNull<$t> {
                ptr.cast()
            }

            #[allow(unreachable_code)]
            fn is_zeroed(_args: &$t) -> bool {
                *_args == $zero

            }
        }

        impl<$($($binder)*)?> crate::Initializer<$t> for $t {
            type Error = core::convert::Infallible;

            fn try_init_into(self, u: Uninit<$t>) -> Result<Init<$t>, Self::Error> {
                Ok(u.write(self))
            }
        }
    };
    ($(=> [$($binder:tt)*])? $t:ty) => {
        impl<$($($binder)*)?> DefaultLayoutProviderFor<$t> for $t {
            type LayoutProvider = PrimitiveLayoutProvider;
        }
        /// SAFETY: is_zeroed never returns zero
        unsafe impl<$($($binder)*)?> LayoutProvider<$t, $t> for PrimitiveLayoutProvider {
            fn layout(_: &$t) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            fn cast(ptr: NonNull<()>, _: &$t) -> NonNull<$t> {
                ptr.cast()
            }

            #[allow(unreachable_code)]
            fn is_zeroed(_args: &$t) -> bool {
                false
            }
        }

        impl<$($($binder)*)?> crate::Initializer<$t> for $t {
            type Error = core::convert::Infallible;

            #[allow(clippy::needless_lifetimes)]
            fn try_init_into<'out>(self, u: Uninit<'out, $t>) -> Result<Init<'out, $t>, Self::Error> {
                Ok(u.write(self))
            }
        }
    };
}

prim!(u8 => 0);
prim!(u16 => 0);
prim!(u32 => 0);
prim!(u64 => 0);
prim!(u128 => 0);
prim!(usize => 0);

prim!(i8 => 0);
prim!(i16 => 0);
prim!(i32 => 0);
prim!(i64 => 0);
prim!(i128 => 0);
prim!(isize => 0);

prim!(f32 => 0.0);
prim!(f64 => 0.0);

prim!(bool => false);
prim!(char => '\0');
prim!(=> [T] *const T => core::ptr::null());
prim!(=> [T] *mut T => core::ptr::null_mut());
prim!(=> [T] *const [T] => core::ptr::slice_from_raw_parts(core::ptr::null(), 0));
prim!(=> [T] *mut [T] => core::ptr::slice_from_raw_parts_mut(core::ptr::null_mut(), 0));
prim!(*const str => core::ptr::slice_from_raw_parts(core::ptr::null::<u8>(), 0) as *const _);
prim!(*mut str => core::ptr::slice_from_raw_parts_mut(core::ptr::null_mut::<u8>(), 0) as *mut _);
prim!(=> [T: ?Sized] PhantomData<T> => PhantomData);

prim!(=> ['a, T: ?Sized] &'a T);
prim!(=> ['a, T: ?Sized] &'a mut T);
prim!(core::time::Duration);
prim!(core::cmp::Ordering);
