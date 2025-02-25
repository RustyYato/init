//! An erasable type with a slice tail

use core::{alloc::Layout, ptr::NonNull};

use crate::{
    layout_provider::{DefaultLayoutProvider, LayoutProvider, SizedLayoutProvider},
    thin::Erasable,
    Ctor, Initializer,
};

/// An erasable type which has a slice tail
#[repr(C)]
pub struct SliceTail<H, T> {
    len: usize,
    header: H,
    tail: [T],
}

// SAFETY: no references are created, and unerase is the inverse of erase
unsafe impl<H, T> Erasable for SliceTail<H, T> {
    unsafe fn erase(ptr: core::ptr::NonNull<Self>) -> core::ptr::NonNull<crate::thin::Erased> {
        ptr.cast()
    }

    unsafe fn unerase(ptr: core::ptr::NonNull<crate::thin::Erased>) -> core::ptr::NonNull<Self> {
        // SAFETY: the `SliceTail`'s first field is the length, and it is Repr(C)
        // so the first field will be the first field in memory
        let len = unsafe { ptr.cast::<usize>().read() };
        let ptr = core::ptr::slice_from_raw_parts_mut(ptr.as_ptr(), len);
        // SAFETY: we got this pointer from a non-null pointer
        unsafe { core::ptr::NonNull::new_unchecked(ptr as _) }
    }
}

/// The layout provider for [`SliceTail`]
pub struct SliceTailLayoutProvider<T, H = SizedLayoutProvider>(H, T);

/// The arguments needed to create a [`SliceTail`]
pub struct SliceTailArgs<H, T> {
    /// The arguments to construct a header
    pub header: H,
    /// The arguments to construct a tail
    pub tail: T,
}

impl<HArgs, TsArgs, H, T> DefaultLayoutProvider<SliceTailArgs<HArgs, TsArgs>> for SliceTail<H, T>
where
    H: DefaultLayoutProvider<HArgs>,
    [T]: DefaultLayoutProvider<TsArgs>,
{
    type LayoutProvider = SliceTailLayoutProvider<
        <[T] as DefaultLayoutProvider<TsArgs>>::LayoutProvider,
        H::LayoutProvider,
    >;
}

// SAFETY:
unsafe impl<LH, LTs, H, T, HArgs, TsArgs>
    LayoutProvider<SliceTail<H, T>, SliceTailArgs<HArgs, TsArgs>>
    for SliceTailLayoutProvider<LTs, LH>
where
    LH: LayoutProvider<H, HArgs>,
    LTs: LayoutProvider<[T], TsArgs>,
{
    fn layout(args: &SliceTailArgs<HArgs, TsArgs>) -> Option<core::alloc::Layout> {
        let tail = LTs::layout(&args.tail)?;
        Some(
            Layout::new::<usize>()
                .extend(Layout::new::<H>())
                .ok()?
                .0
                .extend(tail)
                .ok()?
                .0,
        )
    }

    fn cast(
        ptr: core::ptr::NonNull<()>,
        args: &SliceTailArgs<HArgs, TsArgs>,
    ) -> core::ptr::NonNull<SliceTail<H, T>> {
        let ptr = LTs::cast(ptr, &args.tail).as_ptr();
        // SAFETY: we got this pointer from a non-null pointer
        unsafe { NonNull::new_unchecked(ptr as *mut SliceTail<_, _>) }
    }

    fn is_zeroed(args: &SliceTailArgs<HArgs, TsArgs>) -> bool {
        if !LH::is_zeroed(&args.header) {
            return false;
        }

        if !LTs::is_zeroed(&args.tail) {
            return false;
        }

        let ptr = LTs::cast(core::ptr::NonNull::dangling(), &args.tail).len();
        ptr == 0
    }
}

/// The error type when constructing a SliceTail
pub enum SliceTailError<H, T> {
    /// The header errored
    HeaderError(H),
    /// The tail errored
    TailError(T),
}

impl<H, T, HArgs, TsArgs> Initializer<SliceTail<H, T>> for SliceTailArgs<HArgs, TsArgs>
where
    HArgs: Initializer<H>,
    TsArgs: Initializer<[T]>,
{
    type Error = SliceTailError<HArgs::Error, TsArgs::Error>;

    fn try_init_into(
        self,
        ptr: crate::Uninit<SliceTail<H, T>>,
    ) -> Result<crate::Init<SliceTail<H, T>>, Self::Error> {
        ptr.try_init(crate::init_struct! {
            SliceTail {
                header:crate::try_from_fn(|ptr| ptr.try_init(self.header).map_err(SliceTailError::HeaderError)),
                tail: crate::try_from_fn(|ptr| ptr.try_init(self.tail).map_err(SliceTailError::TailError)),
                len: crate::try_from_fn(|ptr| Ok(ptr.write(tail.as_ref().len()))),
            }
        })
    }
}
