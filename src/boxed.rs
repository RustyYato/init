//! Constructors for boxes

use core::{alloc::Layout, marker::PhantomData, pin::Pin, ptr::NonNull};

use alloc::boxed::Box;

use crate::{layout_provider::LayoutProvider, ptr::Uninit};

/// An initializer for a box
pub struct Emplace<I, L>(I, PhantomData<fn() -> L>);

/// Represents the errors when initializing an element in place in a heap allocation
pub enum EmplaceError<I> {
    /// If the layout could not be computed
    Layout,
    /// If the allocation failed
    Alloc(Layout),
    /// If initialization failed
    Init(I),
}

impl<I> EmplaceError<I> {
    /// Handle the allocate errors, and extract the initialization error
    pub fn handle_alloc_error(self) -> I {
        #[cold]
        #[inline(never)]
        fn handle_alloc_error_(layout: Option<Layout>) -> ! {
            match layout {
                Some(layout) => alloc::alloc::handle_alloc_error(layout),
                None => panic!("Could not compute layout to allocate with"),
            }
        }

        handle_alloc_error_(match self {
            EmplaceError::Layout => None,
            EmplaceError::Alloc(layout) => Some(layout),
            EmplaceError::Init(init) => return init,
        })
    }
}

///
pub fn emplace<I, L>(init: I) -> Emplace<I, L> {
    Emplace(init, PhantomData)
}

///
pub fn emplace_with<T: ?Sized, L, I>(init: I) -> Result<Pin<Box<T>>, EmplaceError<I::Error>>
where
    L: LayoutProvider<T, I>,
    I: crate::Initializer<T>,
{
    struct Allocation {
        ptr: NonNull<u8>,
        layout: Layout,
    }

    impl Drop for Allocation {
        fn drop(&mut self) {
            if self.layout.size() != 0 {
                // SAFETY: Allocation is only constructed with a ptr allocated from the global allocator
                // and is leaked before creating the box
                unsafe { alloc::alloc::dealloc(self.ptr.as_ptr(), self.layout) }
            }
        }
    }

    let layout = L::layout_for(&init).ok_or(EmplaceError::Layout)?;
    let is_zeroed = L::is_zeroed(&init);

    let ptr = if layout.size() == 0 {
        layout.align() as *mut u8
    } else if is_zeroed {
        // SAFETY: the layout has a non-zero size
        unsafe { alloc::alloc::alloc_zeroed(layout) }
    } else {
        // SAFETY: the layout has a non-zero size
        unsafe { alloc::alloc::alloc(layout) }
    };

    let Some(ptr) = NonNull::new(ptr) else {
        return Err(EmplaceError::Alloc(layout))
    };

    // SAFETY: we allocated the pointer with the given layout
    let ptr = unsafe { L::cast(ptr, &init) };

    if !is_zeroed {
        let alloc = Allocation {
            ptr: ptr.cast(),
            layout,
        };

        // SAFETY: The pointer is non-null, aligned, allocated to fit T, and not aliased
        let uninit = unsafe { Uninit::from_raw(ptr.as_ptr()) };

        uninit
            .try_init(init)
            .map_err(EmplaceError::Init)?
            .take_ownership();

        core::mem::forget(alloc);
    }

    // SAFETY: NonNull<T> has the same layout as `Pin<Box<T>>`
    // and doesn't trigger Box's on-move self-reference invalidation
    Ok(unsafe { core::mem::transmute(ptr) })
}

impl<T: ?Sized, I: crate::Initializer<T>, L: LayoutProvider<T, I>> crate::Ctor<Emplace<I, L>>
    for Pin<Box<T>>
{
    type Error = EmplaceError<I::Error>;

    #[inline]
    fn try_init(
        ptr: crate::ptr::Uninit<Self>,
        args: Emplace<I, L>,
    ) -> Result<crate::ptr::Init<Self>, Self::Error> {
        Ok(ptr.write(emplace_with::<T, L, I>(args.0)?))
    }
}

impl<T: ?Sized + Unpin, I: crate::Initializer<T>, L: LayoutProvider<T, I>>
    crate::Ctor<Emplace<I, L>> for Box<T>
{
    type Error = EmplaceError<I::Error>;

    #[inline]
    fn try_init(
        ptr: crate::ptr::Uninit<Self>,
        args: Emplace<I, L>,
    ) -> Result<crate::ptr::Init<Self>, Self::Error> {
        Ok(ptr.write(Pin::into_inner(emplace_with::<T, L, I>(args.0)?)))
    }
}
