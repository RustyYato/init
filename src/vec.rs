//! This contains an extension trait for [`Vec`] to initialize items directly into the spare capacity of a vector

use core::mem::MaybeUninit;

use alloc::vec::Vec;

use crate::{layout_provider::SliceLayoutProvider, ptr::Uninit, Initializer};

/// An extension trait for [`Vec`] to add in place initialization methods
pub trait VecExt {
    /// The type of items stored by the vector
    type Item: Unpin;

    /// initialize the element at position self.len() in place
    ///
    /// # Safety
    ///
    /// The vector's length must not equal it's capacity
    unsafe fn try_emplace_unchecked<I: Initializer<Self::Item>>(
        &mut self,
        initializer: I,
    ) -> Result<(), I::Error>;

    /// initialize the element at position self.len() in place, allocating more space if needed
    fn try_emplace<I: Initializer<Self::Item>>(&mut self, initializer: I) -> Result<(), I::Error>;

    /// push all items in the iterator in place
    fn extend_emplate<I: IntoIterator>(
        &mut self,
        iter: I,
    ) -> Result<(), <I::Item as Initializer<Self::Item>>::Error>
    where
        I::Item: Initializer<Self::Item>;

    /// fill the vector with the slice initiatalizer up to the layout given by the layout provider `L`
    fn try_extend_from_slice_in_place<I, L: SliceLayoutProvider<Self::Item, I>>(
        &mut self,
        slice_initializer: I,
    ) -> Result<(), I::Error>
    where
        I: Initializer<[Self::Item]>;
}

impl<T: Unpin> VecExt for Vec<T> {
    type Item = T;

    fn try_emplace<I: Initializer<Self::Item>>(&mut self, initializer: I) -> Result<(), I::Error> {
        if self.len() == self.capacity() {
            self.reserve(1);
        }

        // SAFETY: if the vector was full, we reserved enough space just above
        unsafe { self.try_emplace_unchecked(initializer) }
    }

    unsafe fn try_emplace_unchecked<I: Initializer<Self::Item>>(
        &mut self,
        initializer: I,
    ) -> Result<(), I::Error> {
        debug_assert_ne!(self.len(), self.capacity());

        let len = self.len();
        let ptr = self.as_mut_ptr();
        // SAFETY: The caller ensures that length isn't out of bounds
        let ptr = unsafe { ptr.add(len) };
        // SAFETY: the vector isn't full, so the slot at element ptr+len is allocated and available
        // for reads and writes
        let uninit = unsafe { Uninit::from_raw(ptr) };
        uninit.try_init(initializer)?.take_ownership();
        let len = self.len();
        // SAFETY: the last element has been initialized
        unsafe { self.set_len(len + 1) };
        Ok(())
    }

    fn extend_emplate<I: IntoIterator>(
        &mut self,
        iter: I,
    ) -> Result<(), <I::Item as Initializer<Self::Item>>::Error>
    where
        I::Item: Initializer<Self::Item>,
    {
        let mut iterator = iter.into_iter();
        while let Some(item) = iterator.next() {
            if self.len() == self.capacity() {
                self.reserve(iterator.size_hint().0);
            }

            // SAFETY: ^^^ ensures that there is enough capacity right above
            unsafe { self.try_emplace_unchecked(item)? }
        }
        Ok(())
    }

    fn try_extend_from_slice_in_place<I, L: SliceLayoutProvider<Self::Item, I>>(
        &mut self,
        slice_initializer: I,
    ) -> Result<(), I::Error>
    where
        I: Initializer<[Self::Item]>,
    {
        let length = L::length(&slice_initializer);

        self.reserve(length);

        let spare = self.spare_capacity_mut();
        // SAFETY: length space was reserved just above, so there is guaranteed to be enough spare capacity
        let spare = unsafe { spare.get_unchecked_mut(..length) };
        let spare: *mut [MaybeUninit<T>] = spare;
        let spare: *mut [T] = spare as _;
        // SAFETY: A vec's spare capacity allocation is aligned, non-null, not aliased, and valid for [T]
        let spare = unsafe { Uninit::from_raw(spare) };
        spare.try_init(slice_initializer)?.take_ownership();
        let len = self.len();
        // SAFETY: the initializer initialized the spare capacity up to length
        unsafe { self.set_len(len + length) }
        Ok(())
    }
}
