//! initialize data directly on the heap

use crate::{
    layout_provider::{DefaultLayoutProvider, LayoutProvider},
    Ctor,
};

use alloc::{
    alloc::{alloc, alloc_zeroed, dealloc, handle_alloc_error},
    boxed::Box,
};
use core::{alloc::Layout, ptr::NonNull};

struct UninitBox {
    ptr: *mut u8,
    layout: Layout,
}

impl Drop for UninitBox {
    fn drop(&mut self) {
        // SAFETY: This type is only constructed after allocating and
        // checking that allocation didn't fail
        unsafe {
            dealloc(self.ptr, self.layout);
        }
    }
}

/// initialize a value directly on the heap
pub fn try_boxed_with<T, I, L>(init: I) -> Result<Box<T>, T::Error>
where
    T: ?Sized + Ctor<I>,
    L: LayoutProvider<T, I>,
{
    let Some(layout) = L::layout(&init) else {
        #[cold]
        #[inline(never)]
        fn handle_layout_error() -> ! {
            panic!("Could not construct layout");
        }

        handle_layout_error()
    };

    let is_zeroed = L::is_zeroed(&init);

    // SAFETY: alloc is only called if the layout has non-zero size
    let ptr = unsafe {
        if layout.size() == 0 {
            layout.align() as *mut u8
        } else if is_zeroed {
            alloc_zeroed(layout)
        } else {
            alloc(layout)
        }
    };

    let Some(ptr) = NonNull::new(ptr) else {
        handle_alloc_error(layout)
    };

    let bx = UninitBox {
        ptr: ptr.as_ptr(),
        layout,
    };

    let ptr = L::cast(ptr.cast(), &init);

    if !is_zeroed {
        // SAFETY: ptr was just allocated with enough space for T
        // LayoutProvider L ensures that the layout is correct
        unsafe { crate::Uninit::from_raw(ptr.as_ptr()) }
            .try_init(init)?
            .take_ownership();
    }

    core::mem::forget(bx);

    // SAFETY: The UninitBox was leaked, so the memory won't be double-freed
    // and the data has been properly initialized by `try_init`
    // is `is_zeroed` is true,
    Ok(unsafe { Box::from_raw(ptr.as_ptr()) })
}

/// initialize a value directly on the heap
pub fn boxed_with<T, I, L>(init: I) -> Box<T>
where
    T: ?Sized + Ctor<I, Error = core::convert::Infallible>,
    L: LayoutProvider<T, I>,
{
    let Ok(bx) = try_boxed_with::<T, I, L>(init);
    bx
}

/// initialize a value directly on the heap
pub fn try_boxed<T, I>(init: I) -> Result<Box<T>, T::Error>
where
    T: ?Sized + Ctor<I> + DefaultLayoutProvider<I>,
{
    try_boxed_with::<T, I, T::LayoutProvider>(init)
}

/// initialize a value directly on the heap
pub fn boxed<T, I>(init: I) -> Box<T>
where
    T: ?Sized + Ctor<I, Error = core::convert::Infallible> + DefaultLayoutProvider<I>,
{
    let Ok(bx) = try_boxed_with::<T, I, T::LayoutProvider>(init);
    bx
}
