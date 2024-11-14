//! initializers for slices

use crate::{
    layout_provider::{DefaultLayoutProvider, LayoutProvider},
    slice_writer::SliceWriter,
    Ctor, Initializer,
};

impl<T: Ctor<()>> Initializer<[T]> for () {
    type Error = T::Error;

    fn try_init_into(self, ptr: crate::Uninit<[T]>) -> Result<crate::Init<[T]>, Self::Error> {
        ptr.try_init(Repeat { init: () })
    }
}

/// Repeat an initializer as many times as necessary to initialize the slice
///
/// see [`repeat`] for details
#[derive(Clone, Copy)]
pub struct CopyFromSlice<'a, T> {
    pub(crate) init: &'a [T],
}

/// Copies the values from the slice directly into the output
pub const fn copy_from_slice<T: Copy>(slice: &[T]) -> CopyFromSlice<T> {
    CopyFromSlice { init: slice }
}

/// The error type for [`CopyFromSlice`]'s [`Initializer`] impl
pub struct CopyFromSliceError {
    /// the length of the slice to copy from into
    pub src_len: usize,
    /// the length of the slice to write into
    pub dest_len: usize,
}

impl<T: Copy> Initializer<[T]> for CopyFromSlice<'_, T> {
    type Error = CopyFromSliceError;

    fn try_init_into(self, mut ptr: crate::Uninit<[T]>) -> Result<crate::Init<[T]>, Self::Error> {
        if self.init.len() != ptr.len() {
            Err(CopyFromSliceError {
                src_len: self.init.len(),
                dest_len: ptr.len(),
            })
        } else {
            // SAFETY: the uninit is not aliased so it doesn't overlap with self.init
            // and we just checked that they have the same lengths
            unsafe {
                ptr.as_mut_ptr()
                    .cast::<T>()
                    .copy_from_nonoverlapping(self.init.as_ptr(), self.init.len())
            };
            // SAFETY: this was copied by above
            Ok(unsafe { ptr.assume_init() })
        }
    }
}

/// Repeat an initializer as many times as necessary to initialize the slice
///
/// see [`repeat`] for details
#[derive(Clone, Copy)]
pub struct Repeat<I> {
    pub(crate) init: I,
}

/// Repeat an initializer as many times as necessary to initialize the slice
pub const fn repeat<I>(init: I) -> Repeat<I> {
    Repeat { init }
}

impl<T: Ctor<I>, I: Clone> Initializer<[T]> for Repeat<I> {
    type Error = T::Error;

    fn try_init_into(self, ptr: crate::Uninit<[T]>) -> Result<crate::Init<[T]>, Self::Error> {
        let mut writer = SliceWriter::new(ptr);

        for _ in 0..writer.remaining_len() {
            // SAFETY: we repeat this for each element of the slice
            unsafe { writer.try_init_unchecked(self.init.clone())? };
        }

        Ok(writer.finish())
    }
}

/// Get initializers from the iterator, and initialize the slice/array using them
///
/// see [`from_iter`] for details
#[derive(Clone, Copy)]
pub struct InitFromIter<I> {
    iter: I,
}

/// Get initializers from the iterator, and initialize the slice/array using them
pub const fn from_iter<I>(iter: I) -> InitFromIter<I> {
    InitFromIter { iter }
}

/// The error type for [`InitFromIter`], specifies if there were not enough elements in the iterator
#[derive(Clone, Copy)]
pub enum InitFromIterError<E> {
    /// If the underlying initializer failed
    Error(E),
    /// IF the iterator ran out of items before initializing all elements of the slice
    NotEnoughItems,
}

impl<T, I: Iterator> Initializer<[T]> for InitFromIter<I>
where
    T: Ctor<I::Item>,
{
    type Error = InitFromIterError<T::Error>;

    fn try_init_into(mut self, ptr: crate::Uninit<[T]>) -> Result<crate::Init<[T]>, Self::Error> {
        let mut writer = SliceWriter::new(ptr);

        for _ in 0..writer.remaining_len() {
            match self.iter.next() {
                // SAFETY: we repeat this for each element of the slice
                Some(init) => unsafe {
                    writer
                        .try_init_unchecked(init)
                        .map_err(InitFromIterError::Error)?
                },
                None => return Err(InitFromIterError::NotEnoughItems),
            }
        }

        Ok(writer.finish())
    }
}

/// A slice layout provider which can be parameterized on another layout provider
pub struct SliceLayoutProvider<L = crate::layout_provider::SizedLayoutProvider>(L);

/// Repeat an initializer as many times as necessary to initialize the slice
#[derive(Clone, Copy)]
pub struct WithLength<I = ()> {
    init: I,
    len: usize,
}

impl WithLength<()> {
    /// Construct a WithLength initializer from an iterator, where the length is the length of the iterator
    pub fn init_from_iter<I: ExactSizeIterator>(iter: I) -> WithLength<InitFromIter<I>> {
        Self::from_init(iter.len(), InitFromIter { iter })
    }

    /// Construct a WithLength initializer from any initializer and length
    pub const fn from_init<I>(len: usize, init: I) -> WithLength<I> {
        WithLength { len, init }
    }
}

impl<I, T: DefaultLayoutProvider<I>> crate::layout_provider::DefaultLayoutProviderFor<[T]>
    for WithLength<Repeat<I>>
{
    type LayoutProvider = SliceLayoutProvider<T::LayoutProvider>;
}
// SAFETY:
// The layout fits [T] with length specified in WithLength,
// and cast returns a slice with the specified length
// is_zeroed simply forwards to L
// L handles the case of cloning I
unsafe impl<T, I, L: LayoutProvider<T, I>>
    crate::layout_provider::LayoutProvider<[T], WithLength<Repeat<I>>> for SliceLayoutProvider<L>
{
    fn layout(args: &WithLength<Repeat<I>>) -> Option<core::alloc::Layout> {
        core::alloc::Layout::array::<T>(args.len).ok()
    }

    fn cast(ptr: core::ptr::NonNull<()>, args: &WithLength<Repeat<I>>) -> core::ptr::NonNull<[T]> {
        core::ptr::NonNull::slice_from_raw_parts(ptr.cast(), args.len)
    }

    fn is_zeroed(args: &WithLength<Repeat<I>>) -> bool {
        L::is_zeroed(&args.init.init)
    }
}

impl<I: Iterator, T: DefaultLayoutProvider<I::Item>>
    crate::layout_provider::DefaultLayoutProviderFor<[T]> for WithLength<InitFromIter<I>>
{
    type LayoutProvider = SliceLayoutProvider<T::LayoutProvider>;
}
// SAFETY:
// The layout fits [T] with length specified in WithLength,
// and cast returns a slice with the specified length
// is_zeroed returns false
// L handles the case of cloning I
unsafe impl<T, I: Iterator, L: LayoutProvider<T, I::Item>>
    crate::layout_provider::LayoutProvider<[T], WithLength<InitFromIter<I>>>
    for SliceLayoutProvider<L>
{
    fn layout(args: &WithLength<InitFromIter<I>>) -> Option<core::alloc::Layout> {
        core::alloc::Layout::array::<T>(args.len).ok()
    }

    fn cast(
        ptr: core::ptr::NonNull<()>,
        args: &WithLength<InitFromIter<I>>,
    ) -> core::ptr::NonNull<[T]> {
        core::ptr::NonNull::slice_from_raw_parts(ptr.cast(), args.len)
    }

    fn is_zeroed(_args: &WithLength<InitFromIter<I>>) -> bool {
        false
    }
}

impl<T: DefaultLayoutProvider<()>> crate::layout_provider::DefaultLayoutProviderFor<[T]>
    for WithLength
{
    type LayoutProvider = SliceLayoutProvider<T::LayoutProvider>;
}
// SAFETY:
// arrays are sized, so layout and cast are trivial
// is_zeroed returns false
// L handles the case of cloning I
unsafe impl<T, L: LayoutProvider<T, ()>> crate::layout_provider::LayoutProvider<[T], WithLength>
    for SliceLayoutProvider<L>
{
    fn layout(args: &WithLength) -> Option<core::alloc::Layout> {
        core::alloc::Layout::array::<T>(args.len).ok()
    }

    fn cast(ptr: core::ptr::NonNull<()>, args: &WithLength) -> core::ptr::NonNull<[T]> {
        core::ptr::NonNull::slice_from_raw_parts(ptr.cast(), args.len)
    }

    fn is_zeroed(_args: &WithLength) -> bool {
        L::is_zeroed(&())
    }
}

// SAFETY:
// arrays are sized, so layout and cast are trivial
// is_zeroed returns false
// L handles the case of cloning I
unsafe impl<T, L: LayoutProvider<T, ()>>
    crate::layout_provider::LayoutProvider<[T], CopyFromSlice<'_, T>> for SliceLayoutProvider<L>
{
    fn layout(args: &CopyFromSlice<T>) -> Option<core::alloc::Layout> {
        Some(core::alloc::Layout::for_value(args.init))
    }

    fn cast(ptr: core::ptr::NonNull<()>, args: &CopyFromSlice<T>) -> core::ptr::NonNull<[T]> {
        core::ptr::NonNull::slice_from_raw_parts(ptr.cast(), args.init.len())
    }

    fn is_zeroed(_args: &CopyFromSlice<T>) -> bool {
        false
    }
}

impl<T, I> Initializer<[T]> for WithLength<I>
where
    [T]: Ctor<I>,
{
    type Error = <[T] as Ctor<I>>::Error;

    fn try_init_into(self, ptr: crate::Uninit<[T]>) -> Result<crate::Init<[T]>, Self::Error> {
        ptr.try_init(self.init)
    }
}
