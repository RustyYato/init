use crate::{
    layout_provider::{LayoutProvider, SizedLayout},
    slice_writer::SliceWriter,
    Ctor,
};

pub struct SliceLayout<L = SliceLayout<SizedLayout>>(L);

// SAFETY: the slice returned from `cast` has the layout specified by `layout_for`
// and slices are zeroable if their element is zeroable
unsafe impl<T, I, L: LayoutProvider<[T], I>> LayoutProvider<[T], InitWithLen<I>>
    for SliceLayout<L>
{
    #[inline]
    fn layout_for(args: &InitWithLen<I>) -> Option<core::alloc::Layout> {
        core::alloc::Layout::array::<T>(args.1).ok()
    }

    #[inline]
    unsafe fn cast(ptr: core::ptr::NonNull<u8>, args: &InitWithLen<I>) -> core::ptr::NonNull<[T]> {
        core::ptr::NonNull::slice_from_raw_parts(ptr.cast(), args.1)
    }

    #[inline]
    fn is_zeroed(args: &InitWithLen<I>) -> bool {
        L::is_zeroed(&args.0)
    }
}

// SAFETY:  since we never return a layout in `layout_for`, it isn't possible to call cast
// and slices are zeroable if their element is zeroable
unsafe impl<T, I, L: LayoutProvider<T, I>> LayoutProvider<[T], CopyArgs<I>> for SliceLayout<L> {
    #[inline]
    fn layout_for(_args: &CopyArgs<I>) -> Option<core::alloc::Layout> {
        None
    }

    #[inline]
    unsafe fn cast(_ptr: core::ptr::NonNull<u8>, _args: &CopyArgs<I>) -> core::ptr::NonNull<[T]> {
        // SAFETY: The caller is not allowed to call cast if `layout_for` doesn't return a
        unsafe { core::hint::unreachable_unchecked() }
    }

    #[inline]
    fn is_zeroed(args: &CopyArgs<I>) -> bool {
        L::is_zeroed(&args.0)
    }
}

// SAFETY:  since we never return a layout in `layout_for`, it isn't possible to call cast
// and slices are zeroable if their element is zeroable
unsafe impl<T, I, L: LayoutProvider<T, I>> LayoutProvider<[T], CloneArgs<I>> for SliceLayout<L> {
    #[inline]
    fn layout_for(_args: &CloneArgs<I>) -> Option<core::alloc::Layout> {
        None
    }

    #[inline]
    unsafe fn cast(_ptr: core::ptr::NonNull<u8>, _args: &CloneArgs<I>) -> core::ptr::NonNull<[T]> {
        // SAFETY: The caller is not allowed to call cast if `layout_for` doesn't return a layout
        unsafe { core::hint::unreachable_unchecked() }
    }

    #[inline]
    fn is_zeroed(args: &CloneArgs<I>) -> bool {
        L::is_zeroed(&args.0)
    }
}

// SAFETY:  since we never return a layout in `layout_for`, it isn't possible to call cast
// and slices are zeroable if their element is zeroable
unsafe impl<T, I, L> LayoutProvider<[T], IterArgs<I>> for SliceLayout<L> {
    #[inline]
    fn layout_for(_args: &IterArgs<I>) -> Option<core::alloc::Layout> {
        None
    }

    #[inline]
    unsafe fn cast(_ptr: core::ptr::NonNull<u8>, _args: &IterArgs<I>) -> core::ptr::NonNull<[T]> {
        // SAFETY: The caller is not allowed to call cast if `layout_for` doesn't return a layout
        unsafe { core::hint::unreachable_unchecked() }
    }

    #[inline]
    fn is_zeroed(_args: &IterArgs<I>) -> bool {
        false
    }
}

pub struct InitWithLen<I>(I, usize);

impl<I> InitWithLen<I> {
    #[inline]
    pub const fn new(len: usize, init: I) -> Self {
        Self(init, len)
    }
}

impl<T, I: crate::Initializer<[T]>> Ctor<InitWithLen<I>> for [T] {
    type Error = I::Error;

    fn try_init(
        ptr: crate::ptr::Uninit<Self>,
        args: InitWithLen<I>,
    ) -> Result<crate::ptr::Init<Self>, Self::Error> {
        assert_eq!(args.1, ptr.len());

        ptr.try_init(args.0)
    }
}

impl<T: Ctor> Ctor for [T] {
    type Error = T::Error;

    fn try_init(
        ptr: crate::ptr::Uninit<Self>,
        _args: (),
    ) -> Result<crate::ptr::Init<Self>, Self::Error> {
        let mut writer = SliceWriter::new(ptr);

        while !writer.try_init(())? {}

        Ok(writer.finish())
    }
}

pub struct CopyArgs<I>(I);

impl<I> CopyArgs<I> {
    #[inline]
    pub const fn new(init: I) -> Self {
        Self(init)
    }

    #[inline]
    pub const fn with_len(self, len: usize) -> InitWithLen<Self> {
        InitWithLen::new(len, self)
    }
}

impl<T, I: crate::Initializer<T> + Copy> Ctor<CopyArgs<I>> for [T] {
    type Error = I::Error;

    fn try_init(
        ptr: crate::ptr::Uninit<Self>,
        args: CopyArgs<I>,
    ) -> Result<crate::ptr::Init<Self>, Self::Error> {
        let mut writer = SliceWriter::new(ptr);

        while !writer.try_init(args.0)? {}

        Ok(writer.finish())
    }
}

pub struct CloneArgs<I>(I);

impl<I> CloneArgs<I> {
    #[inline]
    pub const fn new(init: I) -> Self {
        Self(init)
    }

    #[inline]
    pub const fn with_len(self, len: usize) -> InitWithLen<Self> {
        InitWithLen::new(len, self)
    }
}

impl<T, I: crate::Initializer<T> + Clone> Ctor<CloneArgs<I>> for [T] {
    type Error = I::Error;

    fn try_init(
        ptr: crate::ptr::Uninit<Self>,
        args: CloneArgs<I>,
    ) -> Result<crate::ptr::Init<Self>, Self::Error> {
        let mut writer = SliceWriter::new(ptr);

        if writer.is_complete() {
            return Ok(writer.finish());
        }

        for _ in 1..writer.len() {
            // SAFETY: The writer isn't complete yet
            unsafe { writer.try_init_unchecked(args.0.clone())? }
        }

        // SAFETY: The writer isn't complete yet
        unsafe { writer.try_init_unchecked(args.0)? }

        Ok(writer.finish())
    }
}

pub struct IterArgs<I>(I);

impl<I> IterArgs<I> {
    #[inline]
    pub const fn new(init: I) -> Self {
        Self(init)
    }

    #[inline]
    pub fn with_len(self) -> InitWithLen<Self>
    where
        I: ExactSizeIterator,
    {
        InitWithLen::new(self.0.len(), self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IterInitError<E> {
    NotEnoughElements,
    Init(E),
}

impl<T, I: Iterator> Ctor<IterArgs<I>> for [T]
where
    I::Item: crate::Initializer<T>,
{
    type Error = IterInitError<<I::Item as crate::Initializer<T>>::Error>;

    fn try_init(
        ptr: crate::ptr::Uninit<Self>,
        args: IterArgs<I>,
    ) -> Result<crate::ptr::Init<Self>, Self::Error> {
        let mut writer = SliceWriter::new(ptr);

        if writer.is_complete() {
            return Ok(writer.finish());
        }

        for arg in args.0.take(writer.len()) {
            // SAFETY: The writer isn't complete yet
            unsafe { writer.try_init_unchecked(arg) }.map_err(IterInitError::Init)?
        }

        if writer.is_complete() {
            Ok(writer.finish())
        } else {
            Err(IterInitError::NotEnoughElements)
        }
    }
}
