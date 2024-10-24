//! initializers for arrays

use crate::{
    layout_provider::{DefaultLayoutProvider, LayoutProvider},
    slice, Ctor, Init, Initializer, Uninit,
};

/// Create an initializer for an array from a slice initializer
///
/// see [`from_slice`] for more details
#[derive(Clone, Copy)]
pub struct FromSlice<I>(I);

/// Create an initializer for an array from a slice initializer
pub const fn from_slice<I>(i: I) -> FromSlice<I> {
    FromSlice(i)
}

impl<T, I, const N: usize> Initializer<[T; N]> for FromSlice<I>
where
    [T]: Ctor<I>,
{
    type Error = <[T] as Ctor<I>>::Error;

    fn try_init_into(self, mut ptr: Uninit<[T; N]>) -> Result<Init<[T; N]>, Self::Error> {
        let slice = core::ptr::slice_from_raw_parts_mut(ptr.as_mut_ptr().cast::<T>(), N);
        // SAFETY: This uninit is "re-borrowing" ptr, so it does not alias.
        // It inherits all other safety properties from ptr
        unsafe { Uninit::from_raw(slice) }
            .try_init(self.0)?
            .take_ownership();
        // SAFETY: ptr was just initialized
        Ok(unsafe { ptr.assume_init() })
    }
}

impl<T: Ctor, const N: usize> Initializer<[T; N]> for () {
    type Error = T::Error;

    fn try_init_into(self, ptr: crate::Uninit<[T; N]>) -> Result<crate::Init<[T; N]>, Self::Error> {
        ptr.try_init(from_slice(self))
    }
}

impl<T: Ctor<I>, I: Clone, const N: usize> Initializer<[T; N]> for slice::Repeat<I> {
    type Error = T::Error;

    fn try_init_into(self, ptr: crate::Uninit<[T; N]>) -> Result<crate::Init<[T; N]>, Self::Error> {
        ptr.try_init(from_slice(self))
    }
}

impl<T, I: Iterator, const N: usize> Initializer<[T; N]> for slice::InitFromIter<I>
where
    T: Ctor<I::Item>,
{
    type Error = slice::InitFromIterError<T::Error>;

    fn try_init_into(self, ptr: crate::Uninit<[T; N]>) -> Result<crate::Init<[T; N]>, Self::Error> {
        ptr.try_init(from_slice(self))
    }
}

/// A slice layout provider which can be parameterized on another layout provider
pub struct ArrayLayoutProvider<L = crate::layout_provider::SizedLayoutProvider>(L);

impl<I, T, const N: usize> crate::layout_provider::DefaultLayoutProviderFor<[T; N]> for FromSlice<I>
where
    [T]: DefaultLayoutProvider<I>,
{
    type LayoutProvider = ArrayLayoutProvider<<[T] as DefaultLayoutProvider<I>>::LayoutProvider>;
}
// SAFETY:
// arrays are sized, so layout and cast are trivial
// L handles is_zeroed
// L handles the case of cloning I
unsafe impl<T, I, L: LayoutProvider<[T], I>, const N: usize>
    crate::layout_provider::LayoutProvider<[T; N], FromSlice<I>> for ArrayLayoutProvider<L>
{
    fn layout(_: &FromSlice<I>) -> Option<core::alloc::Layout> {
        Some(core::alloc::Layout::new::<[T; N]>())
    }

    fn cast(ptr: core::ptr::NonNull<()>, _: &FromSlice<I>) -> core::ptr::NonNull<[T; N]> {
        ptr.cast()
    }

    fn is_zeroed(args: &FromSlice<I>) -> bool {
        L::is_zeroed(&args.0)
    }
}

impl<I, T: DefaultLayoutProvider<I>, const N: usize>
    crate::layout_provider::DefaultLayoutProviderFor<[T; N]> for slice::Repeat<I>
{
    type LayoutProvider = ArrayLayoutProvider<T::LayoutProvider>;
}
// SAFETY:
// arrays are sized, so layout and cast are trivial
// is_zeroed simply forwards to L
// L handles the case of cloning I
unsafe impl<T, I, L: LayoutProvider<T, I>, const N: usize>
    crate::layout_provider::LayoutProvider<[T; N], slice::Repeat<I>> for ArrayLayoutProvider<L>
{
    fn layout(_: &slice::Repeat<I>) -> Option<core::alloc::Layout> {
        Some(core::alloc::Layout::new::<[T; N]>())
    }

    fn cast(ptr: core::ptr::NonNull<()>, _: &slice::Repeat<I>) -> core::ptr::NonNull<[T; N]> {
        ptr.cast()
    }

    fn is_zeroed(args: &slice::Repeat<I>) -> bool {
        L::is_zeroed(&args.init)
    }
}

impl<I: Iterator, T: DefaultLayoutProvider<I::Item>, const N: usize>
    crate::layout_provider::DefaultLayoutProviderFor<[T; N]> for slice::InitFromIter<I>
{
    type LayoutProvider = ArrayLayoutProvider<T::LayoutProvider>;
}
// SAFETY:
// arrays are sized, so layout and cast are trivial
// is_zeroed returns false
// L handles the case of cloning I
unsafe impl<T, I: Iterator, L: LayoutProvider<T, I::Item>, const N: usize>
    crate::layout_provider::LayoutProvider<[T; N], slice::InitFromIter<I>>
    for ArrayLayoutProvider<L>
{
    fn layout(_: &slice::InitFromIter<I>) -> Option<core::alloc::Layout> {
        Some(core::alloc::Layout::new::<[T; N]>())
    }

    fn cast(ptr: core::ptr::NonNull<()>, _: &slice::InitFromIter<I>) -> core::ptr::NonNull<[T; N]> {
        ptr.cast()
    }

    fn is_zeroed(_args: &slice::InitFromIter<I>) -> bool {
        false
    }
}

impl<T: DefaultLayoutProvider<()>, const N: usize>
    crate::layout_provider::DefaultLayoutProviderFor<[T; N]> for ()
{
    type LayoutProvider = ArrayLayoutProvider<T::LayoutProvider>;
}
// SAFETY:
// arrays are sized, so layout and cast are trivial
// is_zeroed returns false
// L handles the case of cloning I
unsafe impl<T, L: LayoutProvider<T, ()>, const N: usize>
    crate::layout_provider::LayoutProvider<[T; N], ()> for ArrayLayoutProvider<L>
{
    fn layout(_: &()) -> Option<core::alloc::Layout> {
        Some(core::alloc::Layout::new::<[T; N]>())
    }

    fn cast(ptr: core::ptr::NonNull<()>, _: &()) -> core::ptr::NonNull<[T; N]> {
        ptr.cast()
    }

    fn is_zeroed(_args: &()) -> bool {
        L::is_zeroed(&())
    }
}
