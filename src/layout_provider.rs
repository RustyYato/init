//! This module provides a way to go from initializer arguments to layouts

use core::{alloc::Layout, ptr::NonNull};

/// Specifies the default layout provider to use for a given initializer
pub trait DefaultLayoutProviderFor<T: ?Sized>: Sized {
    /// the layout provider
    type LayoutProvider: LayoutProvider<T, Self>;
}

/// Specifies the default layout provider to use for a given type
pub trait DefaultLayoutProvider<I> {
    /// the layout provider
    type LayoutProvider: LayoutProvider<Self, I>;
}

impl<T: ?Sized, I: DefaultLayoutProviderFor<T>> DefaultLayoutProvider<I> for T {
    type LayoutProvider = I::LayoutProvider;
}

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
///
/// If Args can be cloned, then all clones must produce the same values when applied to any of these functions
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
