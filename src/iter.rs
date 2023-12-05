//! Iterators for `Uninit<[T]>` and `Init<[T]>`

use core::{marker::PhantomData, ptr::NonNull};

use crate::ptr::Uninit;

struct RawIter<T> {
    start: *mut T,
    end: NonNull<T>,
}

impl<T> RawIter<T> {
    const IS_ZST: bool = core::mem::size_of::<T>() == 0;

    /// Create a new raw iterator over the slice
    ///
    ///  Safety:
    ///
    /// The ptr must point to an allocated slice
    pub unsafe fn new(ptr: NonNull<[T]>) -> Self {
        if Self::IS_ZST {
            Self {
                start: ptr.len() as *mut T,
                end: ptr.cast(),
            }
        } else {
            Self {
                start: ptr.as_ptr().cast(),
                // SAFETY: The slice is allocated for `ptr.len()` elements, guaranteed by the caller
                // so getting the 1-past-the-end pointer is safe
                end: unsafe { NonNull::new_unchecked(ptr.as_ptr().cast::<T>().add(ptr.len())) },
            }
        }
    }

    pub fn len(&self) -> usize {
        if Self::IS_ZST {
            self.start as usize
        } else {
            // SAFETY:
            //
            // * Both the starting and other pointer are be either in bounds or one
            //   byte past the end of the same [allocated object].
            //       (done in the constructor)
            // * Both pointers are *derived from* a pointer to the same object.
            //       (done in the constructor)
            // * The distance between the pointers, in bytes, must be an exact multiple
            //   of the size of `T`.
            //      (because we only use `ptr::add` to calculate the next address, this is true)
            // * The distance between the pointers, **in bytes**, cannot overflow an `isize`.
            //      (because the slice is allocated, and all allocations cannot be bigger than isize)
            // * The distance being in bounds cannot rely on "wrapping around" the address space.
            //      (self.start <= self.end)
            unsafe { self.end.as_ptr().offset_from(self.start) as usize }
        }
    }

    pub fn is_empty(&self) -> bool {
        if Self::IS_ZST {
            self.len() == 0
        } else {
            self.end.as_ptr() == self.start
        }
    }

    /// # Safety
    ///
    /// This iterator must not be empty
    pub unsafe fn next_unchecked(&mut self) -> *mut T {
        debug_assert!(!self.is_empty());

        if Self::IS_ZST {
            self.start = (self.start as usize - 1) as *mut T;
            self.end.as_ptr()
        } else {
            let ptr = self.start;
            // SAFETY:
            // * Both the starting and resulting pointer must be either in bounds or one
            //   byte past the end of the same [allocated object].
            //   (since this iterator isn't empty yet, guaranteed by caller, self.start != self.end,
            //    so we have more elemnts in the slice)
            // * The computed offset, **in bytes**, cannot overflow an `isize`.
            //      (because the slice is allocated, and all allocations cannot be bigger than isize)
            // * The offset being in bounds cannot rely on "wrapping around" the address
            //   space. That is, the infinite-precision sum must fit in a `usize`.
            //      (self.start < self.end)
            unsafe { self.start = self.start.add(1) }
            ptr
        }
    }
}

/// An iterator over an uninitialized slice
pub struct UninitIter<'a, T> {
    raw: RawIter<T>,
    inv: PhantomData<Uninit<'a, [T]>>,
}

impl<'a, T> UninitIter<'a, T> {
    /// The remaining number of elements in the iterator
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Does the iterator have any remaining elements
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Get the next element without checking if there is a next elemnt
    ///
    /// # Safety
    ///
    /// this iterator must not be empty
    pub unsafe fn next_unchecked(&mut self) -> Uninit<'a, T> {
        // SAFETY: This iterator isn't empty
        let ptr = unsafe { self.raw.next_unchecked() };
        // SAFETY: this iterator came from a `Uninit<[T]>`, and only yields distinct
        // elements of the slice. So each element is unique.
        unsafe { Uninit::from_raw(ptr) }
    }
}

impl<'a, T> Iterator for UninitIter<'a, T> {
    type Item = Uninit<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            // SAFETY: The iterator isn't empty
            Some(unsafe { self.next_unchecked() })
        }
    }
}

impl<'a, T> IntoIterator for Uninit<'a, [T]> {
    type Item = Uninit<'a, T>;
    type IntoIter = UninitIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        UninitIter {
            // SAFETY: The ptr points to an allocated slice
            raw: unsafe { RawIter::new(self.into_non_null()) },
            inv: PhantomData,
        }
    }
}

#[test]
fn test_raw_iter() {
    // SAFETY: This test is safe
    unsafe {
        let mut iter = RawIter::new(NonNull::from(b"hello world"));

        assert_eq!(iter.len(), 11);
        assert!(!iter.is_empty());
        assert_eq!(*iter.next_unchecked(), b'h');
        assert_eq!(*iter.next_unchecked(), b'e');
        assert_eq!(*iter.next_unchecked(), b'l');
        assert_eq!(*iter.next_unchecked(), b'l');
        assert_eq!(*iter.next_unchecked(), b'o');
        assert_eq!(*iter.next_unchecked(), b' ');
        assert_eq!(*iter.next_unchecked(), b'w');
        assert_eq!(*iter.next_unchecked(), b'o');
        assert_eq!(*iter.next_unchecked(), b'r');
        assert_eq!(*iter.next_unchecked(), b'l');
        assert_eq!(*iter.next_unchecked(), b'd');
        assert_eq!(iter.len(), 0);
        assert!(iter.is_empty());
    }
}
