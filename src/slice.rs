//! initializers for slices

use crate::{layout_provider::LayoutProvider, slice_writer::SliceWriter, Ctor, Initializer};

impl<T: Ctor<()>> Initializer<[T]> for () {
    type Error = T::Error;

    fn try_init_into(self, ptr: crate::Uninit<[T]>) -> Result<crate::Init<[T]>, Self::Error> {
        ptr.try_init(Repeat { init: () })
    }
}

/// Repeat an initializer as many times as necessary to initialize the slice
#[derive(Clone, Copy)]
pub struct Repeat<I> {
    init: I,
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

/// Repeat an initializer as many times as necessary to initialize the slice
#[derive(Clone, Copy)]
pub struct InitFromIter<I> {
    iter: I,
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
pub struct WithLength<I> {
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

impl<T, I> Initializer<[T]> for WithLength<I>
where
    [T]: Ctor<I>,
{
    type Error = <[T] as Ctor<I>>::Error;

    fn try_init_into(self, ptr: crate::Uninit<[T]>) -> Result<crate::Init<[T]>, Self::Error> {
        ptr.try_init(self.init)
    }
}
