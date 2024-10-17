//! This module provides a way to go from initializer arguments to layouts
#![allow(clippy::cmp_null, ambiguous_wide_pointer_comparisons)]

use core::{alloc::Layout, ptr::NonNull};

/// A layout provider specifies...
/// * how to get the layout from the arguments
/// * when it is safe to skip initialization
/// * and how to cast pointers to (potentially) wide pointers
///
/// # Safety
///
/// * layout must give a layout that will fit T
/// * cast must return a pointer that is valid for the associated layout
/// * is_zeroed may only return true if the only thing args does is zero out the memory
pub unsafe trait LayoutProvider<T: ?Sized, Args> {
    /// The layout to allocate a pointer to T with when given the following args
    fn layout(args: &Args) -> Option<Layout>;

    /// Cast an untyped pointer to T (a potentially wide pointer)
    ///
    /// implementors of cast may not read from the pointer
    fn cast(ptr: NonNull<()>, args: &Args) -> NonNull<T>;

    /// Check if args only zeros out the memory
    fn is_zeroed(_args: &Args) -> bool;
}

/// A [`LayoutProvider`] for [`Sized`] types
pub struct SizedLayoutProvider;

// # Safety: is_zeroed always returns false. And the layout of T is always the same for sized T
unsafe impl<T, Args> LayoutProvider<T, Args> for SizedLayoutProvider {
    fn layout(_: &Args) -> Option<Layout> {
        Some(Layout::new::<T>())
    }

    fn cast(ptr: NonNull<()>, _: &Args) -> NonNull<T> {
        ptr.cast()
    }

    fn is_zeroed(_args: &Args) -> bool {
        false
    }
}

/// A [`LayoutProvider`] for [`Sized`] types
pub struct PrimitiveLayoutProvider;

macro_rules! prim {
    ($(=> [$($binder:tt)*])? $t:ty $( => $zero:expr)?) => {
        /// SAFETY: is_zeroed only returns false
        unsafe impl<$($($binder)*)?> LayoutProvider<$t, ()> for PrimitiveLayoutProvider {
            fn layout(_: &()) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            fn cast(ptr: NonNull<()>, _: &()) -> NonNull<$t> {
                ptr.cast()
            }

            fn is_zeroed(_args: &()) -> bool {
                false
            }
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
                $(return *_args == $zero;)?
                false
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
