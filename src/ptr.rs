use core::{marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

use crate::{polyfill, Ctor};

#[cfg(test)]
mod tests;

/// A pointer type which represents a pointer to some uninitialized allocated memory
#[repr(transparent)]
pub struct Uninit<'brand, T: ?Sized> {
    ptr: NonNull<T>,
    brand: PhantomData<fn() -> *mut &'brand ()>,
}

/// A pointer type which represents a pointer to some initialized allocated memory
#[repr(transparent)]
pub struct Init<'brand, T: ?Sized> {
    raw: Uninit<'brand, T>,
}

/// An iterator over a [`Uninit<[T]>`]
pub struct UninitSliceIter<'brand, T> {
    /// The pointer to the next element to return, or the past-the-end location
    /// if the iterator is empty.
    ///
    /// This address will be used for all ZST elements, never changed.
    ptr: NonNull<T>,
    /// For non-ZSTs, the non-null pointer to the past-the-end element.
    ///
    /// For ZSTs, this is `ptr::without_provenance_mut(len)`.
    end_or_len: *mut T,
    _marker: PhantomData<Uninit<'brand, [T]>>,
}

/// An iterator over a [`Init<[T]>`]
pub struct InitSliceIter<'brand, T> {
    iter: UninitSliceIter<'brand, T>,
}

impl<T: ?Sized> Drop for Init<'_, T> {
    fn drop(&mut self) {
        // SAFETY: `Init` represents a raw pointer to an initialized,
        // well aligned, and unique pointer. So it is safe to drop
        unsafe { self.raw.as_mut_ptr().drop_in_place() }
    }
}

impl<'brand, T: ?Sized> Uninit<'brand, T> {
    /// Convert a raw pointer to an [`Uninit`] pointer
    ///
    /// # Safety
    ///
    /// * The pointer must be non-null
    /// * The pointer must be aligned
    /// * The pointer must be allocated with enough room for T
    /// * The pointer must be safe to write to for size_of_val_raw(ptr) bytes
    /// * The pointer must be safe to read from to for size_of_val_raw(ptr) bytes
    /// * The pointer's size_of_val_raw(ptr) must not exceed isize::MAX bytes
    #[inline]
    pub const unsafe fn from_raw(ptr: *mut T) -> Self {
        Self {
            // SAFETY: The caller ensures that the pointer is non-null
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            brand: PhantomData,
        }
    }

    /// Get the underlying raw pointer
    ///
    /// # Safety
    ///
    /// You may not write through this pointer
    pub const fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Get the underlying mutable raw pointer
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Convert this [`Uninit`] into an [`Init`] without checking if it is initialized
    ///
    /// # Safety
    ///
    /// This pointer must point to an initialized value
    pub const unsafe fn assume_init(self) -> Init<'brand, T> {
        Init { raw: self }
    }

    /// Try to initialize self in place with the given arguments
    pub fn try_init<Args>(self, args: Args) -> Result<Init<'brand, T>, T::Error>
    where
        T: Ctor<Args>,
    {
        Ctor::try_init(self, args)
    }

    /// Initialize self in place with the given arguments
    pub fn init<Args>(self, args: Args) -> Init<'brand, T>
    where
        T: Ctor<Args, Error = core::convert::Infallible>,
    {
        let Ok(init) = self.try_init(args);
        init
    }
}

impl<'brand, T> Uninit<'brand, T> {
    /// Write `value` into the pointer, and return the initialized pointer
    pub const fn write(mut self, value: T) -> Init<'brand, T> {
        // SAFETY: as_mut_ptr returns a pointer which is valid for writes
        unsafe { self.as_mut_ptr().write(value) }
        // SAFETY: the pointer is now initialized
        unsafe { self.assume_init() }
    }
}

impl<T> UninitSliceIter<'_, T> {
    const IS_ZST: bool = core::mem::size_of::<T>() == 0;

    const fn new(ptr: NonNull<[T]>) -> Self {
        UninitSliceIter {
            ptr: ptr.cast(),
            end_or_len: if Self::IS_ZST {
                polyfill::without_provenance_mut(ptr.len())
            } else {
                // SAFETY: ptr.len() is correct, so adding it to ptr.as_ptr()
                // will not go past the bounds of the slice
                unsafe { ptr.as_ptr().cast::<T>().add(ptr.len()) }
            },
            _marker: PhantomData,
        }
    }
}

impl<T> Uninit<'_, [T]> {
    /// Get an iterator over [`Uninit<T>`] which points to each element of the slice
    pub fn iter_mut(&mut self) -> UninitSliceIter<'_, T> {
        UninitSliceIter::new(self.ptr)
    }
}

impl<T: ?Sized> AsRef<T> for Init<'_, T> {
    fn as_ref(&self) -> &T {
        self.as_ref()
    }
}

impl<T: ?Sized> Init<'_, T> {
    /// Get a reference to the underlying value
    pub const fn as_ref(&self) -> &T {
        // SAFETY: The pointer is non-null, aligned, allocated, and points to an initialized value
        unsafe { self.raw.ptr.as_ref() }
    }

    /// Get the underlying raw pointer
    pub const fn as_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }

    /// Get the underlying mutable raw pointer
    pub const fn as_mut_ptr(&self) -> *const T {
        self.raw.as_ptr()
    }
}

impl<'brand, T> IntoIterator for Init<'brand, [T]> {
    type IntoIter = InitSliceIter<'brand, T>;
    type Item = Init<'brand, T>;

    fn into_iter(self) -> InitSliceIter<'brand, T> {
        InitSliceIter {
            iter: UninitSliceIter::new(ManuallyDrop::new(self).raw.ptr),
        }
    }
}

const fn iter_assume_init<T>(value: Uninit<T>) -> Init<T> {
    // SAFETY: This is only called in [`InitSliceIter`]
    // and the iterator is only created from a `Init<[T]>`
    unsafe { value.assume_init() }
}

impl<'brand, T> UninitSliceIter<'brand, T> {
    fn len(&self) -> usize {
        if Self::IS_ZST {
            polyfill::addr(self.end_or_len)
        } else {
            // Safety: self.end_or_len come from the same slice as self.ptr
            unsafe { self.end_or_len.offset_from(self.ptr.as_ptr()) as usize }
        }
    }

    fn is_empty(&self) -> bool {
        if Self::IS_ZST {
            self.end_or_len.is_null()
        } else {
            self.ptr.as_ptr() == self.end_or_len
        }
    }

    fn next_unchecked(&mut self) -> Uninit<'brand, T> {
        if Self::IS_ZST {
            self.end_or_len = self.end_or_len.wrapping_byte_sub(1);
            Uninit {
                ptr: self.ptr,
                brand: PhantomData,
            }
        } else {
            let ptr = self.ptr;
            // SAFETY: there is at least one more element in the slice, so
            // add will stay in bounds
            self.ptr = unsafe { self.ptr.add(1) };
            Uninit {
                ptr,
                brand: PhantomData,
            }
        }
    }

    fn next_back_unchecked(&mut self) -> Uninit<'brand, T> {
        if Self::IS_ZST {
            self.end_or_len = self.end_or_len.wrapping_byte_sub(1);
            Uninit {
                ptr: self.ptr,
                brand: PhantomData,
            }
        } else {
            // SAFETY: there is at least one more element in the slice, so
            // sub will stay in bounds, and since it is in bounds end_or_len
            // must be non-null
            unsafe {
                let end_or_len = self.end_or_len.sub(1);
                self.end_or_len = end_or_len;
                Uninit {
                    ptr: NonNull::new_unchecked(end_or_len),
                    brand: PhantomData,
                }
            }
        }
    }

    fn reset(&mut self) {
        if Self::IS_ZST {
            self.end_or_len = core::ptr::null_mut();
        } else {
            self.end_or_len = self.ptr.as_ptr();
        }
    }

    fn fwd_unchecked(&mut self, n: usize) {
        if Self::IS_ZST {
            self.end_or_len = self.end_or_len.wrapping_byte_sub(n);
        } else {
            // SAFETY: n < the number of remaining elements in the slice
            self.ptr = unsafe { self.ptr.add(n) };
        }
    }

    fn bck_unchecked(&mut self, n: usize) {
        if Self::IS_ZST {
            self.end_or_len = self.end_or_len.wrapping_byte_sub(n);
        } else {
            // SAFETY: n < the number of remaining elements in the slice
            self.end_or_len = unsafe { self.end_or_len.sub(n) };
        }
    }
}

impl<T> ExactSizeIterator for UninitSliceIter<'_, T> {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<'brand, T> Iterator for UninitSliceIter<'brand, T> {
    type Item = Uninit<'brand, T>;

    #[allow(unstable_name_collisions)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.next_unchecked())
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.reset();
            None
        } else {
            self.fwd_unchecked(n);
            Some(self.next_unchecked())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for UninitSliceIter<'_, T> {
    #[allow(unstable_name_collisions)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            Some(self.next_back_unchecked())
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            self.reset();
            None
        } else {
            self.bck_unchecked(n);
            Some(self.next_back_unchecked())
        }
    }
}

impl<T> ExactSizeIterator for InitSliceIter<'_, T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}
impl<'brand, T> Iterator for InitSliceIter<'brand, T> {
    type Item = Init<'brand, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(iter_assume_init)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for InitSliceIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(iter_assume_init)
    }
}
