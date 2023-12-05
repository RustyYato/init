//! an interface for calcualting the layout and generically casting pointers to possibly unsized types

use core::{alloc::Layout, pin::Pin, ptr::NonNull};

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
    fn layout_for(_: &Args) -> Option<Layout> {
        Some(Layout::new::<T>())
    }

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
            fn layout_for(_: &()) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

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
            fn layout_for(_: &$t) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

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
            fn layout_for(_: &&$t) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

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
            fn layout_for(_: &&mut $t) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

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
            fn layout_for(_: &Pin<&$t>) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

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
            fn layout_for(_: &Pin<&mut $t>) -> Option<Layout> {
                Some(Layout::new::<$t>())
            }

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

primitive! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
    char = '\0' bool = false
    f32 = 0.0 f64 = 0.0
}
