//! an interface for calcualting the layout and generically casting pointers to possibly unsized types

use core::{alloc::Layout, num, pin::Pin, ptr::NonNull};

/// A trait that describes types which can calculate the layout and
/// cast pointers based on ctor initializers
///
/// # Safety
///
/// cast must not change the pointee of `ptr`
/// if `is_zeroed` returns true, it must be safe to skip initialization and
/// zeroing out the memory must have the same effects as initialization
pub unsafe trait LayoutProvider<T: ?Sized, Args = ()> {
    /// Returns the layout of `T` for the given arguments
    fn layout_for(args: &Args) -> Option<Layout>;

    /// Casts the untyped pointer allocated for with the same layout as `Self::layout_for(args)`
    ///
    /// # Safety
    ///
    /// * The pointer must have been allocated with the same layout as `Self::layout_for(args)`
    unsafe fn cast(ptr: NonNull<u8>, args: &Args) -> NonNull<T>;

    /// Returns true iff it is safe to replace initialization with just zeroing out memory
    #[inline]
    fn is_zeroed(_args: &Args) -> bool {
        false
    }
}

/// A layout provider for sized types
pub struct SizedLayout;

/// SAFETY: cast doesn't change the pointee of `ptr`
unsafe impl<T, Args> LayoutProvider<T, Args> for SizedLayout {
    #[inline]
    fn layout_for(_: &Args) -> Option<Layout> {
        Some(Layout::new::<T>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &Args) -> NonNull<T> {
        ptr.cast()
    }
}

/// A layout provider for primitive types
pub struct PrimitiveLayout;

macro_rules! pick {
    ($a:literal $b:literal) => {
        $a
    };
    ($a:literal) => {
        $a
    };
}

macro_rules! primitive {
    ($($t:ident $(= $val:literal)?)*) => {$(
        // SAFETY: sized types can always just use cast
        unsafe impl LayoutProvider<$t> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &()) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<$t> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(_args: &()) -> bool {
                true
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<$t, $t> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &$t) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &$t) -> NonNull<$t> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(args: &$t) -> bool {
                *args == pick!($($val)? 0)
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<$t, &$t> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &&$t) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &&$t) -> NonNull<$t> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(args: &&$t) -> bool {
                **args == pick!($($val)? 0)
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<$t, &mut $t> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &&mut $t) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &&mut $t) -> NonNull<$t> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(args: &&mut $t) -> bool {
                **args == pick!($($val)? 0)
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<$t, Pin<&$t>> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &Pin<&$t>) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &Pin<&$t>) -> NonNull<$t> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(args: &Pin<&$t>) -> bool {
                **args == pick!($($val)? 0)
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<$t, Pin<&mut $t>> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &Pin<&mut $t>) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &Pin<&mut $t>) -> NonNull<$t> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(args: &Pin<&mut $t>) -> bool {
                **args == pick!($($val)? 0)
            }
        }
    )*};
}

macro_rules! nz_primitive {
    ($($t:ident)*) => {$(
        // SAFETY: sized types can always just use cast
        unsafe impl LayoutProvider<num::$t> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &()) -> Option<Layout> {
                Some(Layout::new::<num::$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<num::$t> {
                ptr.cast()
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<num::$t, num::$t> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &num::$t) -> Option<Layout> {
                Some(Layout::new::<num::$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &num::$t) -> NonNull<num::$t> {
                ptr.cast()
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<num::$t, &num::$t> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &&num::$t) -> Option<Layout> {
                Some(Layout::new::<num::$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &&num::$t) -> NonNull<num::$t> {
                ptr.cast()
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<num::$t, &mut num::$t> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &&mut num::$t) -> Option<Layout> {
                Some(Layout::new::<num::$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &&mut num::$t) -> NonNull<num::$t> {
                ptr.cast()
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<num::$t, Pin<&num::$t>> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &Pin<&num::$t>) -> Option<Layout> {
                Some(Layout::new::<num::$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &Pin<&num::$t>) -> NonNull<num::$t> {
                ptr.cast()
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<num::$t, Pin<&mut num::$t>> for PrimitiveLayout {
            #[inline]
            fn layout_for(_: &Pin<&mut num::$t>) -> Option<Layout> {
                Some(Layout::new::<num::$t>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &Pin<&mut num::$t>) -> NonNull<num::$t> {
                ptr.cast()
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<Option<num::$t>> for NicheLayout {
            #[inline]
            fn layout_for(_: &()) -> Option<Layout> {
                Some(Layout::new::<Option<num::$t>>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<Option<num::$t>> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(_args: &()) -> bool {
                true
            }
        }

        // SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
        unsafe impl LayoutProvider<Option<num::$t>, Option<num::$t>> for NicheLayout {
            #[inline]
            fn layout_for(_: &Option<num::$t>) -> Option<Layout> {
                Some(Layout::new::<Option<num::$t>>())
            }

            #[inline]
            unsafe fn cast(ptr: NonNull<u8>, _: &Option<num::$t>) -> NonNull<Option<num::$t>> {
                ptr.cast()
            }

            #[inline]
            fn is_zeroed(args: &Option<num::$t>) -> bool {
                args.is_none()
            }
        }
    )*};
}

primitive! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
    char = '\0' bool = false
    f32 = 0.0 f64 = 0.0
}

nz_primitive! {
    NonZeroU8 NonZeroU16 NonZeroU32 NonZeroU64 NonZeroU128 NonZeroUsize
    NonZeroI8 NonZeroI16 NonZeroI32 NonZeroI64 NonZeroI128 NonZeroIsize
}

/// The layout provider for Option<T>
pub struct OptionLayout;
/// The layout provider for Option<&T>, Option<&mut T>, Option<NonNull<T>>
pub struct NicheLayout;

// SAFETY: sized types can always just use cast
unsafe impl<T> LayoutProvider<Option<T>> for OptionLayout {
    #[inline]
    fn layout_for(_: &()) -> Option<Layout> {
        Some(Layout::new::<Option<T>>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<Option<T>> {
        ptr.cast()
    }
}

// SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
unsafe impl<'a, T: ?Sized + SizedOrSlice> LayoutProvider<Option<&'a T>> for NicheLayout {
    #[inline]
    fn layout_for(_: &()) -> Option<Layout> {
        Some(Layout::new::<Option<&T>>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<Option<&'a T>> {
        ptr.cast()
    }

    #[inline]
    fn is_zeroed(_args: &()) -> bool {
        true
    }
}

// SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
unsafe impl<'a, T: ?Sized + SizedOrSlice> LayoutProvider<Option<&'a mut T>> for NicheLayout {
    #[inline]
    fn layout_for(_: &()) -> Option<Layout> {
        Some(Layout::new::<Option<&T>>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<Option<&'a mut T>> {
        ptr.cast()
    }

    #[inline]
    fn is_zeroed(_args: &()) -> bool {
        true
    }
}

// SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
unsafe impl<'a, T: ?Sized + SizedOrSlice> LayoutProvider<Option<Pin<&'a T>>> for NicheLayout {
    #[inline]
    fn layout_for(_: &()) -> Option<Layout> {
        Some(Layout::new::<Option<&T>>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<Option<Pin<&'a T>>> {
        ptr.cast()
    }

    #[inline]
    fn is_zeroed(_args: &()) -> bool {
        true
    }
}

// SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
unsafe impl<'a, T: ?Sized + SizedOrSlice> LayoutProvider<Option<Pin<&'a mut T>>> for NicheLayout {
    #[inline]
    fn layout_for(_: &()) -> Option<Layout> {
        Some(Layout::new::<Option<&T>>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<Option<Pin<&'a mut T>>> {
        ptr.cast()
    }

    #[inline]
    fn is_zeroed(_args: &()) -> bool {
        true
    }
}

// SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
unsafe impl<T: ?Sized + SizedOrSlice> LayoutProvider<Option<NonNull<T>>> for NicheLayout {
    #[inline]
    fn layout_for(_: &()) -> Option<Layout> {
        Some(Layout::new::<Option<&T>>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<Option<NonNull<T>>> {
        ptr.cast()
    }

    #[inline]
    fn is_zeroed(_args: &()) -> bool {
        true
    }
}

// SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
unsafe impl<T: ?Sized + SizedOrSlice> LayoutProvider<Option<alloc::boxed::Box<T>>> for NicheLayout {
    #[inline]
    fn layout_for(_: &()) -> Option<Layout> {
        Some(Layout::new::<Option<&T>>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<Option<alloc::boxed::Box<T>>> {
        ptr.cast()
    }

    #[inline]
    fn is_zeroed(_args: &()) -> bool {
        true
    }
}

// SAFETY: sized types can always just use cast, is_zeroed is compatible with `Ctor`
unsafe impl<T: ?Sized + SizedOrSlice> LayoutProvider<Option<Pin<alloc::boxed::Box<T>>>>
    for NicheLayout
{
    #[inline]
    fn layout_for(_: &()) -> Option<Layout> {
        Some(Layout::new::<Option<&T>>())
    }

    #[inline]
    unsafe fn cast(ptr: NonNull<u8>, _: &()) -> NonNull<Option<Pin<alloc::boxed::Box<T>>>> {
        ptr.cast()
    }

    #[inline]
    fn is_zeroed(_args: &()) -> bool {
        true
    }
}

/// # Safety
///
/// This trait may only be implemented on sized types and slices
pub unsafe trait SizedOrSlice {}

/// SAFETY: This trait may only be implemented on sized types and slices
unsafe impl<T> SizedOrSlice for T {}
/// SAFETY: This trait may only be implemented on sized types and slices
unsafe impl<T> SizedOrSlice for [T] {}
